use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use std::path::Path;
use syn::{LitStr, parse_macro_input};

/// Compile-time JSON to Layout conversion macro
///
/// Usage: `layout_from_json!("path/to/layout.json")`
///
/// This macro reads a JSON file at compile time and generates
/// the corresponding Layout struct initialization code.
/// It automatically recompiles when the JSON file changes.
#[proc_macro]
pub fn layout_from_json(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let file_path = input.value();

    // Validate at compile time
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let full_path = Path::new(&manifest_dir).join(&file_path);

    let json_content = fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to read JSON file {}: {}", file_path, e));

    // Validate JSON structure at compile time
    let layout: cluster_core::models::Layout = serde_json::from_str(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON in {}: {}", file_path, e));

    // Generate initialization code
    let layout_code = generate_layout_code(&layout);

    // Generate code that includes the file for change tracking
    let code = quote! {
        {
            // This ensures Cargo tracks the file but we don't actually use it
            const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #file_path));

            // Return the pre-validated layout
            #layout_code
        }
    };

    code.into()
}

fn generate_layout_code(layout: &cluster_core::models::Layout) -> proc_macro2::TokenStream {
    let f0_code = generate_cluster_code(&layout.f0);
    let f1_code = generate_cluster_code(&layout.f1);
    let f1b_code = generate_cluster_code(&layout.f1b);
    let f2_code = generate_cluster_code(&layout.f2);
    let f4_code = generate_cluster_code(&layout.f4);
    let f6_code = generate_cluster_code(&layout.f6);

    quote! {
        cluster_core::models::Layout {
            f0: #f0_code,
            f1: #f1_code,
            f1b: #f1b_code,
            f2: #f2_code,
            f4: #f4_code,
            f6: #f6_code,
        }
    }
}

fn generate_cluster_code(cluster: &cluster_core::models::Cluster) -> proc_macro2::TokenStream {
    let message = &cluster.message;
    let name = &cluster.name;

    // Generate attributes
    let attributes = cluster.attributes.iter().map(|attr| match attr {
        cluster_core::types::Attribute::Piscine => {
            quote! { cluster_core::types::Attribute::Piscine }
        }
        cluster_core::types::Attribute::Exam => quote! { cluster_core::types::Attribute::Exam },
        cluster_core::types::Attribute::Silent => quote! { cluster_core::types::Attribute::Silent },
        cluster_core::types::Attribute::Event => quote! { cluster_core::types::Attribute::Event },
        cluster_core::types::Attribute::Closed => quote! { cluster_core::types::Attribute::Closed },
    });

    // Generate seats
    let seats = cluster.seats.iter().map(|seat| {
        let id = &seat.id;
        let kind = match seat.kind {
            cluster_core::types::Kind::Mac => quote! { cluster_core::types::Kind::Mac },
            cluster_core::types::Kind::Lenovo => quote! { cluster_core::types::Kind::Lenovo },
            cluster_core::types::Kind::Dell => quote! { cluster_core::types::Kind::Dell },
            cluster_core::types::Kind::Flex => quote! { cluster_core::types::Kind::Flex },
        };
        let status = match seat.status {
            cluster_core::types::Status::Free => quote! { cluster_core::types::Status::Free },
            cluster_core::types::Status::Taken => quote! { cluster_core::types::Status::Taken },
            cluster_core::types::Status::Reported => {
                quote! { cluster_core::types::Status::Reported }
            }
            cluster_core::types::Status::Broken => quote! { cluster_core::types::Status::Broken },
        };
        let x = seat.x;
        let y = seat.y;

        quote! {
            cluster_core::models::Seat {
                id: #id.try_into().expect("Invalid seat ID"),
                kind: #kind,
                status: #status,
                x: #x,
                y: #y,
            }
        }
    });

    // Generate zones
    let zones = cluster.zones.iter().map(|zone| {
        let zone_name = &zone.name;
        let zone_attributes = zone.attributes.iter().map(|attr| match attr {
            cluster_core::types::Attribute::Piscine => {
                quote! { cluster_core::types::Attribute::Piscine }
            }
            cluster_core::types::Attribute::Exam => quote! { cluster_core::types::Attribute::Exam },
            cluster_core::types::Attribute::Silent => {
                quote! { cluster_core::types::Attribute::Silent }
            }
            cluster_core::types::Attribute::Event => {
                quote! { cluster_core::types::Attribute::Event }
            }
            cluster_core::types::Attribute::Closed => {
                quote! { cluster_core::types::Attribute::Closed }
            }
        });
        let x = zone.x;
        let y = zone.y;

        quote! {
            cluster_core::models::Zone {
                name: #zone_name.try_into().expect("Invalid zone name"),
                attributes: {
                    let mut attrs = cluster_core::types::AttributeVec::new();
                    #(
                        let _ = attrs.push(#zone_attributes);
                    )*
                    attrs
                },
                x: #x,
                y: #y,
            }
        }
    });

    quote! {
        cluster_core::models::Cluster {
            message: #message.try_into().expect("Invalid message"),
            name: #name.try_into().expect("Invalid cluster name"),
            attributes: {
                let mut attrs = cluster_core::types::AttributeVec::new();
                #(
                    let _ = attrs.push(#attributes);
                )*
                attrs
            },
            seats: {
                let mut seats = cluster_core::models::SeatVec::new();
                #(
                    let _ = seats.push(#seats);
                )*
                seats
            },
            zones: {
                let mut zones = cluster_core::models::ZoneVec::new();
                #(
                    let _ = zones.push(#zones);
                )*
                zones
            },
        }
    }
}
