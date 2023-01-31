#![allow(non_snake_case)]
#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
use std::env;
use std::fs::File;
use std::io::Read;

use syn::FnArg;

struct Me {
    field: bool,
}

enum You {
    Variant1,
    Variant2,
}

fn funky() {
    ()
}

trait Naughty {}
pub trait Nice {}

fn main() -> anyhow::Result<()> {
    let mut file = File::open("./src/main.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let ast = syn::parse_file(&content)?;
    if let Some(shebang) = ast.shebang {
        println!("{}", shebang);
    }
    println!("{} items", ast.items.len());

    ast.items.iter().for_each(|it| match it {
        syn::Item::Const(ItemConst) => {
            dbg!(&ItemConst.ident);
            println!("constant");
        }
        syn::Item::Enum(ItemEnum) => {
            dbg!(&ItemEnum.ident);
            println!("enum");
        }
        syn::Item::ExternCrate(ItemExternCrate) => {
            dbg!(&ItemExternCrate.ident);
            println!("extern crate");
        }
        syn::Item::Fn(ItemFn) => {
            dbg!(&ItemFn.sig.ident);
            println!("function");
        }
        syn::Item::ForeignMod(ItemForeignMod) => {
            println!("foreign module");
        }
        syn::Item::Impl(ItemImpl) => {
            //dbg!(&ItemImpl.self_ty);
            println!("impl");
        }
        syn::Item::Macro(ItemMacro) => {
            //dbg!(&ItemMacro.mac.path);
            println!("macro");
        }
        syn::Item::Macro2(ItemMacro2) => {
            //dbg!(&ItemMacro2.mac.path);
            println!("macro2");
        }
        syn::Item::Mod(ItemMod) => {
            //dbg!(&ItemMod.content);
            println!("module");
        }
        syn::Item::Static(ItemStatic) => {
            dbg!(&ItemStatic.ident);
            println!("static");
        }
        syn::Item::Struct(ItemStruct) => {
            dbg!(&ItemStruct.ident);
            println!("structure");
        }
        syn::Item::Trait(ItemTrait) => {
            dbg!(&ItemTrait.ident);
            println!("trait");
        }
        syn::Item::TraitAlias(ItemTraitAlias) => {
            dbg!(&ItemTraitAlias.ident);
            println!("trait alias");
        }
        syn::Item::Type(ItemType) => {
            dbg!(&ItemType.ident);
            println!("type");
        }
        syn::Item::Union(ItemUnion) => {
            dbg!(&ItemUnion.ident);
            println!("union");
        }
        syn::Item::Use(ItemUse) => {
            //dbg!(&ItemUse.path);
            println!("use");
        }
        syn::Item::Verbatim(TokenStream) => {
            dbg!(&TokenStream);
            println!("verbatim");
        }
        _ => (),
    });

    Ok(())
}
