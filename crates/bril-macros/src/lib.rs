use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Const;
use syn::{bracketed, parse_macro_input, LitInt, Token};

/// The instruction macro takes the following values which need to
/// be key value inputs:
///     - op: The operation (mandatory)
///     - args: The arguments to the operations (optional)
///     - ty: The type of the input (optional)
///     - value: The value of the input (optional)
///     - dest: The variable destination of the operation (optional)
#[proc_macro]
pub fn instruction(input: TokenStream) -> TokenStream {
    let instruction = parse_macro_input!(input as Instruction);

    let mut output = proc_macro2::TokenStream::new();
    instruction.to_tokens(&mut output);

    output.into()
}

/// Util macro for easy syn::Error generation
macro_rules! error {
    ($span: expr, $msg: expr) => {
        syn::Error::new($span, $msg)
    };
}

/// Wrapper around a bril Instruction. Used for parsing.
#[derive(Default, Debug)]
struct Instruction(bril::types::Instruction);

mod kw {
    syn::custom_keyword!(op);
    syn::custom_keyword!(args);
    syn::custom_keyword!(ty);
    syn::custom_keyword!(value);
    syn::custom_keyword!(dest);
}

impl Parse for Instruction {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(error!(input.span(), "expected at least an 'op' field"));
        }

        let mut has_operation = false;
        let mut instruction = Instruction::default();

        // Keep parsing while there are values in the stream
        while !input.is_empty() {
            if input.peek(kw::op) {
                if has_operation {
                    return Err(error!(input.span(), "operation already set"));
                }
                instruction.0.op = Operation::parse(input)?.0;
                has_operation = true;
            } else if input.peek(kw::args) {
                if instruction.0.args.is_some() {
                    return Err(error!(input.span(), "args already set"));
                }
                instruction.0.args = Some(Args::parse(input)?.0);
            } else if input.peek(kw::value) {
                if instruction.0.value.is_some() {
                    return Err(error!(input.span(), "value already set"));
                }
                instruction.0.value = Some(input.parse::<Value>()?.0);
            } else if input.peek(kw::ty) {
                if instruction.0.r#type.is_some() {
                    return Err(error!(input.span(), "type already set"));
                }
                instruction.0.r#type = Some(input.parse::<Type>()?.0);
            } else if input.peek(kw::dest) {
                if instruction.0.dest.is_some() {
                    return Err(error!(input.span(), "dest already set"));
                }
                instruction.0.dest = Some(input.parse::<Dest>()?.0)
            } else {
                return Err(error!(
                    input.span(),
                    format!("unexpected attribute {input}")
                ));
            }

            // Parse the comma that separates all the values
            // Don't propagate error in case we are at the end of
            // the macro
            let _ = input.parse::<Token![,]>();
        }

        // Operation is the only value required
        if !has_operation {
            return Err(error!(input.span(), "'op' attribute needs to be set"));
        }

        // Before returning, we verify if the instruction is a valid instruction
        if !instruction.0.is_valid() {
            return Err(error!(input.span(), "invalid instruction"));
        }

        Ok(instruction)
    }
}

impl ToTokens for Instruction {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let none = quote!(None);

        let op = Ident::new(&format!("{:?}", self.0.op), Span::call_site());
        let op = quote!(bril::types::Operation::#op);

        let args = self
            .0
            .args
            .as_ref()
            .map(|args| {
                let args = args.iter().map(|a| quote!(#a.to_string()));
                quote!(Some(vec![#(#args,)*]))
            })
            .unwrap_or_else(|| none.clone());

        let ty = self
            .0
            .r#type
            .as_ref()
            .map(|t| {
                let t = Ident::new(&format!("{t:?}"), Span::call_site());
                quote!(Some(bril::types::Type::#t))
            })
            .unwrap_or_else(|| none.clone());

        let value = self
            .0
            .value
            .as_ref()
            .map(|v| quote!(Some(#v)))
            .unwrap_or_else(|| none.clone());

        let dest = self
            .0
            .dest
            .as_ref()
            .map(|d| quote!(Some(#d.to_string())))
            .unwrap_or_else(|| none.clone());

        let instr = quote!(
            bril::types::Instruction {
                op: #op,
                args: #args,
                value: #value,
                dest: #dest,
                r#type: #ty
            }
        );

        tokens.extend(instr);
    }
}

struct Operation(bril::types::Operation);

impl Parse for Operation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<kw::op>()?;
        let _ = input.parse::<Token![=]>()?;
        let op = input.parse::<Ident>();

        // If the parsing failed, try to parse the `const` keyword.
        // This is the only operation that needs special attention
        // because `const` is a reserved keyword
        if op.is_err() {
            let _ = input.parse::<Const>()?;
            return Ok(Self(bril::types::Operation::Const));
        }

        let op = op?.to_string();
        Ok(Self(bril::types::Operation::from_str(&op).map_err(
            |_| error!(input.span(), format!("expected valid operation, got {op}")),
        )?))
    }
}

struct Value(u32);

impl Parse for Value {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<kw::value>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse::<LitInt>()?.base10_parse()?;

        Ok(Self(value))
    }
}

struct Type(bril::types::Type);

impl Parse for Type {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<kw::ty>()?;
        let _ = input.parse::<Token![=]>()?;
        let ty = input.parse::<Ident>()?.to_string();

        Ok(Self(bril::types::Type::from_str(&ty).map_err(|_| {
            error!(input.span(), format!("expected valid type, got {ty}"))
        })?))
    }
}

struct Dest(bril::types::Var);

impl Parse for Dest {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<kw::dest>()?;
        let _ = input.parse::<Token![=]>()?;
        let ty = input.parse::<Ident>()?.to_string();

        Ok(Self(ty))
    }
}

struct Args(bril::types::Args);

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<kw::args>()?;
        let _ = input.parse::<Token![=]>()?;

        // Parse the values between square brackets
        let content;
        bracketed!(content in input);

        let args = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
        let args = args.into_iter().map(|i| i.to_string()).collect();

        Ok(Self(args))
    }
}
