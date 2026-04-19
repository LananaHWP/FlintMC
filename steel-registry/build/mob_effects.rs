use std::fs;

use heck::ToShoutySnakeCase;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
struct MobEffectEntry {
    id: i32,
    name: String,
    category: String,
    color: i32,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=build_assets/mob_effects.json");

    let content = fs::read_to_string("build_assets/mob_effects.json").unwrap();
    let mut effects: Vec<MobEffectEntry> = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse mob_effects.json: {}", e));

    effects.sort_by_key(|a| a.id);

    let mut stream = TokenStream::new();

    stream.extend(quote! {
        use crate::mob_effect::{MobEffect, MobEffectCategory, MobEffectRegistry};
    });

    let mut register_stream = TokenStream::new();
    for effect in &effects {
        let ident = Ident::new(&effect.name.to_shouty_snake_case(), Span::call_site());
        let name = &effect.name;
        let category = match effect.category.as_str() {
            "BENEFICIAL" => quote! { MobEffectCategory::Beneficial },
            "HARMFUL" => quote! { MobEffectCategory::Harmful },
            "NEUTRAL" => quote! { MobEffectCategory::Neutral },
            _ => panic!("Unknown effect category: {}", effect.category),
        };
        let color = Literal::i32_unsuffixed(effect.color);
        let effect_id = Literal::i32_unsuffixed(effect.id);
        let translation_key = format!("effect.minecraft.{}", effect.name);

        stream.extend(quote! {
            pub static #ident: &MobEffect = &MobEffect::new(
                #effect_id,
                "minecraft",
                #name,
                #translation_key,
                #category,
                #color,
            );
        });

        register_stream.extend(quote! {
            registry.register(#ident);
        });
    }

    stream.extend(quote! {
        pub fn register_mob_effects(registry: &mut MobEffectRegistry) {
            #register_stream
        }
    });

    stream
}