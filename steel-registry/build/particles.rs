use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=build_assets/particles.json");

    let particles: BTreeMap<String, i32> =
        serde_json::from_str(&fs::read_to_string("build_assets/particles.json").unwrap())
            .expect("Failed to parse particles.json");

    let consts: TokenStream = particles
        .iter()
        .map(|(name, value)| {
            let name = format_ident!("{}", name);
            quote! {
                pub const #name: i32 = #value;
            }
        })
        .collect();

    quote!(
        //! Particle type constants matching vanilla Minecraft's ParticleType registry.
        //!
        //! These IDs are used with the `CLevelParticles` packet to spawn particles.
        //!
        //! Particle types with additional data (like block, item, dust) require extra parameters
        //! in the particle data field of the packet.

        #consts
    )
}