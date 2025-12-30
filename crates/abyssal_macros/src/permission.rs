use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Arm, Ident, ItemEnum, ItemImpl, Token, Variant, braced, parse::Parse, punctuated::Punctuated,
};

#[derive(Clone, Debug)]
enum Node {
    Leaf { name: Ident },
    Branch { name: Ident, nodes: Vec<Node> },
}

impl Node {
    pub fn name(&self) -> Ident {
        match self {
            Node::Leaf { name } => name.clone(),
            Node::Branch { name, .. } => name.clone(),
        }
    }
}

#[derive(Clone, Debug)]
struct Tree {
    name: Ident,
    nodes: Vec<Node>,
}

impl Parse for Node {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let peek = input.lookahead1();
        if peek.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            Ok(Self::Leaf { name })
        } else if peek.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let content;
            braced!(content in input);

            let mut nodes: Vec<Node> = vec![];

            while !content.is_empty() {
                nodes.push(content.parse::<Node>()?);
            }
            input.parse::<Token![;]>()?;

            Ok(Self::Branch { name, nodes })
        } else {
            Err(input.error(format!(
                "Unexpected token! Expected `;` (for a leaf node) or `=>` (for a branch node)"
            )))
        }
    }
}

impl Parse for Tree {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=>]>()?;
        let content;
        braced!(content in input);

        let mut nodes: Vec<Node> = vec![];

        while !content.is_empty() {
            nodes.push(content.parse::<Node>()?);
        }

        Ok(Self { name, nodes })
    }
}

fn render_branch(
    parent: Option<Ident>,
    enum_ident: Ident,
    name: Ident,
    nodes: Vec<Node>,
) -> manyhow::Result<(Variant, Vec<ItemEnum>, Vec<ItemImpl>, Vec<ItemImpl>)> {
    let mut node_variants: Punctuated<Variant, Token![,]> = Punctuated::new();
    let mut node_enums: Vec<ItemEnum> = vec![];
    let mut node_try_froms: Vec<ItemImpl> = vec![];
    let mut node_intos: Vec<ItemImpl> = vec![];
    let mut try_from_match_arms: Punctuated<Arm, Token![,]> = Punctuated::new();
    let mut into_match_arms: Punctuated<Arm, Token![,]> = Punctuated::new();

    for node in nodes.clone() {
        let (e_variant, e_enums, e_try_froms, e_intos) = node.render(Some(
            parent
                .clone()
                .and_then(|p| Some(format_ident!("{p}{name}")))
                .unwrap_or(name.clone()),
        ))?;
        node_variants.push(e_variant);
        node_enums.extend(e_enums);
        node_try_froms.extend(e_try_froms);
        node_intos.extend(e_intos);

        let node_id = node.name().to_string().to_case(Case::Snake);
        let node_enum_ident = format_ident!("{}Permission", node.name());
        let node_ident = node.name();

        match node.clone() {
            Node::Leaf { .. } => {
                try_from_match_arms.push(syn::parse2(quote! {#node_id => Ok(Self::#node_ident)})?);
                into_match_arms.push(syn::parse2(
                    quote! {#enum_ident::#node_ident => #node_id.to_string()},
                )?);
            }
            Node::Branch { .. } => {
                try_from_match_arms.push(syn::parse2(
        quote! {#node_id => Ok(Self::#node_ident(if let Some(remain) = remainder {#node_enum_ident::try_from(remain)?} else {#node_enum_ident::default()}))},
    )?);
                into_match_arms.push(syn::parse2(quote! {#enum_ident::#node_ident(child) => if let #node_enum_ident::All = child {#node_id.to_string()} else {format!("{}.{}", #node_id.to_string(), String::from(child))}})?);
            }
        };
    }

    node_enums.push(syn::parse2(quote! {
        #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Default)]
        #[serde(try_from = "String", into = "String")]
        pub enum #enum_ident {
            #[default]
            All,
            #node_variants
        }
    })?);

    node_try_froms.push(syn::parse2(quote! {
        impl TryFrom<String> for #enum_ident {
            type Error = crate::Error;
            fn try_from(value: String) -> crate::Result<Self> {
                let (current, remainder) = value.split_once(".").and_then(|(c, r)| Some((c.to_string(), Some(r.to_string())))).unwrap_or((value, None));
                match current.as_str() {
                    #try_from_match_arms,
                    other => Err(crate::Error::unknown_permission(other))
                }
            }
        }
    })?);

    node_intos.push(syn::parse2(quote! {
        impl From<#enum_ident> for String {
            fn from(value: #enum_ident) -> Self {
                match value {
                    #into_match_arms,
                    #enum_ident::All => String::from("all")
                }
            }
        }
    })?);

    Ok((
        syn::parse2(quote! {#name(#enum_ident)})?,
        node_enums,
        node_try_froms,
        node_intos,
    ))
}

impl Node {
    pub fn render(
        &self,
        parent: Option<Ident>,
    ) -> manyhow::Result<(Variant, Vec<ItemEnum>, Vec<ItemImpl>, Vec<ItemImpl>)> {
        match self {
            Node::Leaf { name } => Ok((
                syn::parse2::<Variant>(quote! {#name})?,
                vec![],
                vec![],
                vec![],
            )),
            Node::Branch { name, nodes } => render_branch(
                parent.clone(),
                format_ident!("{name}Permission"),
                name.clone(),
                nodes.clone(),
            ),
        }
    }
}

pub fn impl_permissions(input: TokenStream) -> manyhow::Result<TokenStream> {
    let Tree { name, nodes } = syn::parse2::<Tree>(input)?;
    let (_, enums, try_froms, intos) = render_branch(None, name.clone(), name, nodes)?;

    Ok(quote! {
        #(#enums)*
        #(#try_froms)*
        #(#intos)*
    })
}
