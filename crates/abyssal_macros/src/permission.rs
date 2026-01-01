use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Arm, Expr, ExprArray, ExprLit, Ident, ItemEnum, ItemImpl, Lit, Token, Variant, braced, bracketed, parse::{Parse, ParseBuffer, discouraged::Speculative}, punctuated::Punctuated
};

#[derive(Clone, Debug)]
enum Node {
    Leaf { name: Ident, metadata: Metadata },
    Branch { name: Ident, nodes: Vec<Node>, metadata: Metadata },
    StringLeaf { name: Ident, parameter: String, metadata: Metadata },
    StringBranch { name: Ident, nodes: Vec<Node>, parameter: String, metadata: Metadata }
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

#[derive(Clone, Debug, Default)]
struct Metadata {
    pub comment: Option<String>,
    pub depends: Vec<String>
}

impl Parse for Metadata {
    fn parse(content: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut meta = Self::default();
        while !content.is_empty() {
            let key = content.parse::<Ident>()?;
            content.parse::<Token![=]>()?;
            let value = content.parse::<Expr>()?;
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
            match key.to_string().as_str() {
                "comment" => {
                    if let Expr::Lit(ExprLit {lit: Lit::Str(lit_str), ..}) = value {
                        meta.comment = Some(lit_str.value());
                    } else {
                        Err(content.error("Expected a string literal for `comment`"))?;
                    }
                },
                "depends" => {
                    if let Expr::Array(ExprArray {elems, ..}) = value {
                        let mut depends_on = Vec::<String>::new();
                        for elem in elems {
                            if let Expr::Lit(ExprLit {lit: Lit::Str(lit_str), ..}) = elem {
                                depends_on.push(lit_str.value());
                            } else {
                                Err(content.error("Expected string literal"))?;
                            }
                        }

                        meta.depends = depends_on;
                    } else {
                        Err(content.error("Expected an array of subpaths"))?;
                    }
                },
                other => {
                    Err(content.error(format!("Unexpected metadata key \"{other}\"")))?;
                }
            }
        }

        Ok(meta)
    }
}

impl Metadata {
    pub fn definitely_parse(input: &ParseBuffer<'_>) -> syn::Result<syn::Result<Metadata>> {
        let forked = input.fork();
        let content;
        bracketed!(content in forked);
        input.advance_to(&forked);
        Ok(content.parse::<Metadata>())
    }
}

#[derive(Clone, Debug)]
struct Tree {
    pub name: Ident,
    pub nodes: Vec<Node>,
}

impl Parse for Node {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            Ok(Self::Leaf { name, metadata: Metadata::default() })
        } else if let Ok(metadata) = Metadata::definitely_parse(&input) {
            input.parse::<Token![;]>()?;
            Ok(Self::Leaf { name, metadata: metadata? })
        } else if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;

            let ahead = input.fork();
            if let Ok(parameter) = ahead.parse::<syn::LitStr>().map(|v| v.value()) {
                input.advance_to(&ahead);
                if input.peek(Token![;]) {
                    input.parse::<Token![;]>()?;
                    Ok(Self::StringLeaf { name, parameter, metadata: Metadata::default() })
                } else if let Ok(metadata) = Metadata::definitely_parse(&input) {
                    input.parse::<Token![;]>()?;
                    Ok(Self::StringLeaf { name, parameter, metadata: metadata? })
                } else {
                    let content;
                    braced!(content in input);
                    let mut nodes: Vec<Node> = vec![];

                    while !content.is_empty() {
                        nodes.push(content.parse::<Node>()?);
                    }

                    let metadata = Metadata::definitely_parse(&input).unwrap_or(Ok(Metadata::default()))?;

                    input.parse::<Token![;]>()?;

                    Ok(Self::StringBranch { name, nodes, parameter, metadata })
                }
            } else {
                let content;
                braced!(content in input);

                let mut nodes: Vec<Node> = vec![];

                while !content.is_empty() {
                    nodes.push(content.parse::<Node>()?);
                }
                
                let metadata = Metadata::definitely_parse(&input).unwrap_or(Ok(Metadata::default()))?;

                input.parse::<Token![;]>()?;

                Ok(Self::Branch { name, nodes, metadata })
            }
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
    metadata: Metadata
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
        #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, Default, Eq, PartialEq)]
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

    let name_repr = name.to_string().to_case(Case::Snake);
    let Metadata {comment, depends, ..} = metadata;
    let comment_repr = if let Some(c) = comment {
        syn::parse2::<Expr>(quote! {Some(#c.to_string())})?
    } else {
        syn::parse2::<Expr>(quote! {None})?
    };
    let depends_on_repr = syn::parse2::<Expr>(quote! {vec![#(#depends.to_string()),*]})?;
    if let Some(param) = parameter {
        node_methods.push(syn::parse2(quote! {
            impl #methods_ident for #enum_ident {
                fn describe() -> #description_ident {
                    #description_ident::StringBranch{
                        name: #name_repr.to_string(),
                        nodes: vec![#described_nodes],
                        parameter: #param.to_string(),
                        comment: #comment_repr,
                        depends_on: #depends_on_repr
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
                        comment: #comment_repr,
                        depends_on: #depends_on_repr
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
            Node::Leaf { name, metadata } => Ok((
                syn::parse2::<Variant>(quote! {#name})?,
                vec![],
                vec![],
                vec![],
                vec![],
                {
                    let name_repr = name.to_string().to_case(Case::Snake);
                    let Metadata {comment, depends, ..} = metadata;
                    let comment_repr = if let Some(c) = comment {
                        syn::parse2::<Expr>(quote! {Some(#c.to_string())})?
                    } else {
                        syn::parse2::<Expr>(quote! {None})?
                    };
                    let depends_on_repr = syn::parse2::<Expr>(quote! {vec![#(#depends.to_string()),*]})?;
                    let out = syn::parse2::<Expr>(quote! {
                        #description_ident::Leaf{
                            name: #name_repr.to_string(),
                            comment: #comment_repr,
                            depends_on: #depends_on_repr
                        }
                    })?;
                    out
                }
            )),
            Node::Branch { name, nodes, metadata } => render_branch(
                root.clone(),
                parent.clone(),
                format_ident!("{name}Permission"),
                name,
                nodes,
                false,
                None,
                metadata
            ),
            Node::StringLeaf { name, parameter, metadata } => Ok((
                syn::parse2::<Variant>(quote! {#name {parameter: Option<String>}})?,
                vec![],
                vec![],
                vec![],
                vec![],
                {
                    let name_repr = name.to_string().to_case(Case::Snake);
                    let Metadata {comment, depends, ..} = metadata;
                    let comment_repr = if let Some(c) = comment {
                        syn::parse2::<Expr>(quote! {Some(#c.to_string())})?
                    } else {
                        syn::parse2::<Expr>(quote! {None})?
                    };
                    let depends_on_repr = syn::parse2::<Expr>(quote! {vec![#(#depends.to_string()),*]})?;
                    syn::parse2::<Expr>(quote! {
                        #description_ident::StringLeaf{
                            name: #name_repr.to_string(),
                            parameter: #parameter.to_string(),
                            comment: #comment_repr,
                            depends_on: #depends_on_repr
                        }
                    })?
                }
            )),
            Node::StringBranch { name, nodes, parameter, metadata } => render_branch(
                root.clone(),
                parent.clone(),
                format_ident!("{name}Permission"),
                name,
                nodes,
                true,
                Some(parameter),
                metadata
            )
        }
    }
}

pub fn impl_permissions(input: TokenStream) -> manyhow::Result<TokenStream> {
    let Tree { name, nodes } = syn::parse2::<Tree>(input)?;
    let (_, enums, try_froms, intos, methods, _) = render_branch(name.clone(), None, name.clone(), name.clone(), nodes, false, None, Metadata::default())?;

    let methods_ident = format_ident!("{name}Methods");
    let description_ident = format_ident!("{name}Description");

    Ok(quote! {
        #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
        #[serde(rename_all = "snake_case", tag = "node_type")]
        pub enum #description_ident {
            Leaf {
                name: String,
                comment: Option<String>,
                depends_on: Vec<String>
            },
            Branch {
                name: String,
                nodes: Vec<#description_ident>,
                comment: Option<String>,
                depends_on: Vec<String>
            },
            StringLeaf {
                name: String,
                parameter: String,
                comment: Option<String>,
                depends_on: Vec<String>
            },
            StringBranch {
                name: String,
                nodes: Vec<#description_ident>,
                parameter: String,
                comment: Option<String>,
                depends_on: Vec<String>
            }
        }

        impl #description_ident {
            pub fn name(&self) -> String {
                match self.clone() {
                    Self::Leaf {name, ..} => name,
                    Self::Branch {name, ..} => name,
                    Self::StringLeaf {name, ..} => name,
                    Self::StringBranch {name, ..} => name,
                }
            }

            pub fn comment(&self) -> Option<String> {
                match self.clone() {
                    Self::Leaf {comment, ..} => comment,
                    Self::Branch {comment, ..} => comment,
                    Self::StringLeaf {comment, ..} => comment,
                    Self::StringBranch {comment, ..} => comment,
                }
            }

            pub fn depends_on(&self) -> Vec<String> {
                match self.clone() {
                    Self::Leaf {depends_on, ..} => depends_on,
                    Self::Branch {depends_on, ..} => depends_on,
                    Self::StringLeaf {depends_on, ..} => depends_on,
                    Self::StringBranch {depends_on, ..} => depends_on,
                }
            }

            pub fn nodes(&self) -> Option<Vec<Self>> {
                match self.clone() {
                    Self::Leaf {..} => None,
                    Self::Branch {nodes, ..} => Some(nodes),
                    Self::StringLeaf {..} => None,
                    Self::StringBranch {nodes, ..} => Some(nodes),
                }
            }

            pub fn parameter(&self) -> Option<String> {
                match self.clone() {
                    Self::Leaf {..} => None,
                    Self::Branch {..} => None,
                    Self::StringLeaf {parameter, ..} => Some(parameter),
                    Self::StringBranch {parameter, ..} => Some(parameter),
                }
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
