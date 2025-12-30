use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Arm, Expr, Ident, ItemEnum, ItemImpl, Token, Variant, braced, parse::{Parse, discouraged::Speculative}, punctuated::Punctuated
};

#[derive(Clone, Debug)]
enum Node {
    Leaf { name: Ident, comment: Option<String> },
    Branch { name: Ident, nodes: Vec<Node>, comment: Option<String> },
    StringLeaf { name: Ident, parameter: String, comment: Option<String> },
    StringBranch { name: Ident, nodes: Vec<Node>, parameter: String, comment: Option<String> }
}

impl Node {
    pub fn name(&self) -> Ident {
        match self {
            Node::Leaf { name,.. } => name.clone(),
            Node::Branch { name, .. } => name.clone(),
            Node::StringLeaf { name,.. } => name.clone(),
            Node::StringBranch { name, .. } => name.clone(),
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
        let ahead = input.fork();
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            Ok(Self::Leaf { name, comment: None })
        } else if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;

            let ahead = input.fork();
            if let Ok(parameter) = ahead.parse::<syn::LitStr>().map(|v| v.value()) {
                input.advance_to(&ahead);
                let ahead = input.fork();
                if input.peek(Token![;]) {
                    input.parse::<Token![;]>()?;
                    Ok(Self::StringLeaf { name, parameter, comment: None })
                } else if let Ok(comment) = ahead.parse::<syn::LitStr>() {
                    input.advance_to(&ahead);
                    input.parse::<Token![;]>()?;
                    Ok(Self::StringLeaf { name, parameter, comment: Some(comment.value()) })
                } else {
                    let content;
                    braced!(content in input);
                    let mut nodes: Vec<Node> = vec![];

                    while !content.is_empty() {
                        nodes.push(content.parse::<Node>()?);
                    }

                    let ahead = input.fork();
                    let comment = if let Ok(comment) = ahead.parse::<syn::LitStr>() {
                        input.advance_to(&ahead);
                        Some(comment.value())
                    } else {
                        None
                    };

                    input.parse::<Token![;]>()?;

                    Ok(Self::StringBranch { name, nodes, parameter, comment })
                }
            } else {
                let content;
                braced!(content in input);

                let mut nodes: Vec<Node> = vec![];

                while !content.is_empty() {
                    nodes.push(content.parse::<Node>()?);
                }
                
                let ahead = input.fork();
                let comment = if let Ok(comment) = ahead.parse::<syn::LitStr>() {
                    input.advance_to(&ahead);
                    Some(comment.value())
                } else {
                    None
                };

                input.parse::<Token![;]>()?;

                Ok(Self::Branch { name, nodes, comment })
            }
        } else if let Ok(comment) = ahead.parse::<syn::LitStr>() {
            input.advance_to(&ahead);
            input.parse::<Token![;]>()?;
            Ok(Self::Leaf { name, comment: Some(comment.value()) })
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
    root: Ident,
    parent: Option<Ident>,
    enum_ident: Ident,
    name: Ident,
    nodes: Vec<Node>,
    is_string_branch: bool,
    parameter: Option<String>,
    comment: Option<String>
) -> manyhow::Result<(Variant, Vec<ItemEnum>, Vec<ItemImpl>, Vec<ItemImpl>, Vec<ItemImpl>, Expr)> {
    let methods_ident = format_ident!("{root}Methods");
    let description_ident = format_ident!("{root}Description");
    
    let mut node_variants: Punctuated<Variant, Token![,]> = Punctuated::new();
    let mut node_enums: Vec<ItemEnum> = vec![];
    let mut node_try_froms: Vec<ItemImpl> = vec![];
    let mut node_intos: Vec<ItemImpl> = vec![];
    let mut node_methods: Vec<ItemImpl> = vec![];
    let mut try_from_match_arms: Punctuated<Arm, Token![,]> = Punctuated::new();
    let mut into_match_arms: Punctuated<Arm, Token![,]> = Punctuated::new();
    let mut described_nodes: Punctuated<Expr, Token![,]> = Punctuated::new();

    for node in nodes.clone() {
        let (e_variant, e_enums, e_try_froms, e_intos, e_methods, e_description) = node.render(root.clone(), Some(
            parent
                .clone()
                .and_then(|p| Some(format_ident!("{p}{name}")))
                .unwrap_or(name.clone()),
        ))?;
        node_variants.push(e_variant);
        node_enums.extend(e_enums);
        node_try_froms.extend(e_try_froms);
        node_intos.extend(e_intos);
        node_methods.extend(e_methods);
        described_nodes.push(e_description);

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
            quote! {#node_id => Ok(Self::#node_ident{child: if let Some(remain) = remainder {#node_enum_ident::try_from(remain)?} else {#node_enum_ident::default()}})},
                )?);
                into_match_arms.push(syn::parse2(quote! {#enum_ident::#node_ident{child} => format!("{}.{}", #node_id.to_string(), String::from(child))})?);
            }
            Node::StringLeaf { .. } => {
                try_from_match_arms.push(syn::parse2(
            quote! {#node_id => Ok(Self::#node_ident{parameter: if let Some(remain) = remainder {Some(remain)} else {None}})},
                )?);
                into_match_arms.push(syn::parse2(quote! {#enum_ident::#node_ident{parameter} => if let Some(child_value) = parameter {format!("{}.{}", #node_id.to_string(), child_value)} else {format!("{}.*", #node_id.to_string())}})?);
            }
            Node::StringBranch { .. } => {
                try_from_match_arms.push(syn::parse2(
            quote! {#node_id => {
                        if let Some(remain) = remainder {
                            if let Some((param, next)) = remain.split_once(".") {
                                Ok(Self::#node_ident{parameter: if param == "*" {None} else {Some(param.to_string())}, child: #node_enum_ident::try_from(next.to_string())?})
                            } else {
                                Ok(Self::#node_ident{parameter: if remain.as_str() == "*" {None} else {Some(remain)}, child: #node_enum_ident::default()})
                            }
                        } else {
                            Ok(Self::#node_ident{parameter: None, child: #node_enum_ident::default()})
                        }
                    }},
                )?);
                into_match_arms.push(syn::parse2(quote! {#enum_ident::#node_ident{parameter: param, child} => {
                    if let Some(existing) = param {
                        if let #node_enum_ident::All = child {
                            format!("{}.{}.*", #node_id.to_string(), existing)
                        } else {
                            format!("{}.{}.{}", #node_id.to_string(), existing, String::from(child))
                        }
                    } else {
                        if let #node_enum_ident::All = child {
                            format!("{}.*", #node_id.to_string())
                        } else {
                            format!("{}.*.{}", #node_id.to_string(), String::from(child))
                        }
                    }
                }})?);
            }
        };
    }

    node_enums.push(syn::parse2(quote! {
        #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Default)]
        #[serde(rename_all = "snake_case", tag = "name")]
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
                let (current, remainder) = value.split_once(".").and_then(|(c, r)| Some((c.to_string(), if r == "*" {None} else {Some(r.to_string())}))).unwrap_or((value, None));
                match current.as_str() {
                    #try_from_match_arms,
                    "*" => Ok(Self::All),
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
                    #enum_ident::All => String::from("*")
                }
            }
        }
    })?);

    let name_repr = name.to_string();
    let comment_repr = if let Some(c) = comment {
        syn::parse2::<Expr>(quote! {Some(#c.to_string())})?
    } else {
        syn::parse2::<Expr>(quote! {None})?
    };
    if let Some(param) = parameter {
        node_methods.push(syn::parse2(quote! {
            impl #methods_ident for #enum_ident {
                fn describe() -> #description_ident {
                    #description_ident::StringBranch{
                        name: #name_repr.to_string(),
                        nodes: vec![#described_nodes],
                        parameter: #param.to_string(),
                        comment: #comment_repr
                    }
                }
            }
        })?);
    } else {
        node_methods.push(syn::parse2(quote! {
            impl #methods_ident for #enum_ident {
                fn describe() -> #description_ident {
                    #description_ident::Branch{
                        name: #name_repr.to_string(),
                        nodes: vec![#described_nodes],
                        comment: #comment_repr
                    }
                }
            }
        })?);
    }

    Ok((
        if is_string_branch {syn::parse2(quote! {#name {parameter: Option<String>, child: #enum_ident}})?} else {syn::parse2(quote! {#name {child: #enum_ident}})?},
        node_enums,
        node_try_froms,
        node_intos,
        node_methods,
        syn::parse2::<Expr>(quote! {
            #enum_ident::describe()
        })?
    ))
}

impl Node {
    pub fn render(
        &self,
        root: Ident,
        parent: Option<Ident>,
    ) -> manyhow::Result<(Variant, Vec<ItemEnum>, Vec<ItemImpl>, Vec<ItemImpl>, Vec<ItemImpl>, Expr)> {
        let description_ident = format_ident!("{root}Description");
        match self.clone() {
            Node::Leaf { name, comment } => Ok((
                syn::parse2::<Variant>(quote! {#name})?,
                vec![],
                vec![],
                vec![],
                vec![],
                {
                    let comment_repr = if let Some(c) = comment {
                        syn::parse2::<Expr>(quote! {Some(#c.to_string())})?
                    } else {
                        syn::parse2::<Expr>(quote! {None})?
                    };
                    let name_repr = name.to_string();
                    let out = syn::parse2::<Expr>(quote! {
                        #description_ident::Leaf{
                            name: #name_repr.to_string(),
                            comment: #comment_repr
                        }
                    })?;
                    out
                }
            )),
            Node::Branch { name, nodes, comment } => render_branch(
                root.clone(),
                parent.clone(),
                format_ident!("{name}Permission"),
                name,
                nodes,
                false,
                None,
                comment
            ),
            Node::StringLeaf { name, parameter, comment } => Ok((
                syn::parse2::<Variant>(quote! {#name {parameter: Option<String>}})?,
                vec![],
                vec![],
                vec![],
                vec![],
                {
                    let comment_repr = if let Some(c) = comment {
                        syn::parse2::<Expr>(quote! {Some(#c.to_string())})?
                    } else {
                        syn::parse2::<Expr>(quote! {None})?
                    };
                    let name_repr = name.to_string();
                    syn::parse2::<Expr>(quote! {
                        #description_ident::StringLeaf{
                            name: #name_repr.to_string(),
                            parameter: #parameter.to_string(),
                            comment: #comment_repr
                        }
                    })?
                }
            )),
            Node::StringBranch { name, nodes, parameter, comment } => render_branch(
                root.clone(),
                parent.clone(),
                format_ident!("{name}Permission"),
                name,
                nodes,
                true,
                Some(parameter),
                comment
            )
        }
    }
}

pub fn impl_permissions(input: TokenStream) -> manyhow::Result<TokenStream> {
    let Tree { name, nodes } = syn::parse2::<Tree>(input)?;
    let (_, enums, try_froms, intos, methods, _) = render_branch(name.clone(), None, name.clone(), name.clone(), nodes, false, None, None)?;

    let methods_ident = format_ident!("{name}Methods");
    let description_ident = format_ident!("{name}Description");

    Ok(quote! {
        #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
        #[serde(rename_all = "snake_case", tag = "node_type")]
        pub enum #description_ident {
            Leaf {
                name: String,
                comment: Option<String>
            },
            Branch {
                name: String,
                nodes: Vec<#description_ident>,
                comment: Option<String>
            },
            StringLeaf {
                name: String,
                parameter: String,
                comment: Option<String>
            },
            StringBranch {
                name: String,
                nodes: Vec<#description_ident>,
                parameter: String,
                comment: Option<String>
            }
        }

        pub trait #methods_ident {
            fn describe() -> #description_ident;
        }

        #(#enums)*
        #(#try_froms)*
        #(#intos)*
        #(#methods)*
    })
}
