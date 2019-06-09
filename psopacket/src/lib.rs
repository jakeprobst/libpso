#![recursion_limit="128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};
use quote::quote;

#[proc_macro_attribute]
pub fn pso_packet(attr: TokenStream, item: TokenStream) -> TokenStream {
    let arg = parse_macro_input!(attr as syn::LitInt);
    let pkt_cmd = arg.value() as u16;
    
    let parsed = parse_macro_input!(item as ItemStruct);

    let mut has_flag: bool = false;
    let mut from_bytes = Vec::new();
    let mut as_bytes = Vec::new();
    let mut dbg_write_vars = Vec::new();
    let mut partialeq = Vec::new();
    
    for (i, f) in parsed.fields.iter().enumerate() {
        if let Some(ident) = &f.ident {
            let ident_str = ident.to_string();

            match &f.ty {
                syn::Type::Array(arr) => {
                    let array_length = if let syn::Expr::Lit(lit) = &arr.len {
                        if let syn::Lit::Int(int) = &lit.lit {
                            int.value() as usize
                        }
                        else {
                            return syn::Error::new(arr.bracket_token.span, "unknown array size").to_compile_error().into();
                        }
                    } else {
                        return syn::Error::new(arr.bracket_token.span, "unknown array size").to_compile_error().into();
                    };
                    match *arr.elem {
                        syn::Type::Path(ref path) => {
                            let ty = path.path.segments[0].ident.to_string();
                            if ty.as_str() == "u8_str" {
                                dbg_write_vars.push(quote! {
                                    match std::str::from_utf8(&self.#ident) {
                                        Ok(v) => write!(f, "    {}: {:?}\n", #ident_str, v).unwrap(),
                                        Err(_) => write!(f, "    {}: {:?}\n", #ident_str, self.#ident.iter()).unwrap()
                                    }
                                });
                            }
                            else {
                                dbg_write_vars.push(quote! {
                                    write!(f, "    {}: {:?}\n", #ident_str, self.#ident.iter()).unwrap();
                                });
                            }
                            //dbg_write_vars.push(quote! {
                                //write!(f, "    {}: {:?}\n", #ident_str, self.#ident.iter()).unwrap();
                                /*match std::str::from_utf8(&self.#ident) {
                                    Ok(v) => write!(f, "    {}: {:?}\n", #ident_str, v).unwrap(),
                                    Err(_) => write!(f, "    {}: {:?}\n", #ident_str, self.#ident.iter()).unwrap()
                                }*/
                                //write!(f, "    {}: {:?}\n", #ident_str, var_as_str).unwrap();
                            //});
                            as_bytes.push(quote! {
                                for f in self.#ident.iter() {
                                    buf.extend_from_slice(&f.to_le_bytes())
                                }
                            });
                            match ty.as_str() {
                                "u8" | "u8_str" => {
                                    from_bytes.push(quote! {
                                        #ident: {
                                            let mut b: [u8; #array_length] = [0; #array_length];
                                            if let Ok(len) = cur.read(&mut b) {
                                                if len != #array_length {
                                                    return Err(PacketParseError::NotEnoughBytes);
                                                }
                                            }
                                            else {
                                                return Err(PacketParseError::NotEnoughBytes);
                                            };
                                            b
                                        },
                                    });
                                },
                                _ => {
                                    return syn::Error::new(path.path.segments[0].ident.span(), "type not supported")
                                        .to_compile_error().into();
                                }
                            }
                            partialeq.push(quote! {
                                if self.#ident[..] != other.#ident[..] {
                                    return false;
                                }
                            });
                        }
                        _ => {
                            panic!("why");
                        }
                    }
                },
                syn::Type::Path(path) => {
                    dbg_write_vars.push(quote! {
                        write!(f, "    {}: {:?}\n", #ident_str, self.#ident).unwrap();
                    });
                    as_bytes.push(quote! {
                        buf.extend_from_slice(&self.#ident.to_le_bytes());
                    });
                    let ty = path.path.segments[0].ident.to_string();
                    match ty.as_str() {
                        "u8" | "u8_str" => {
                            from_bytes.push(quote! {
                                #ident: {
                                    let mut b: [u8; 1] = [0; 1];
                                    if let Ok(len) = cur.read(&mut b) {
                                        if len != 1 {
                                            return Err(PacketParseError::NotEnoughBytes);
                                        }
                                    }
                                    else {
                                        return Err(PacketParseError::NotEnoughBytes);
                                    };
                                    b[0]
                                },
                            });
                        },
                        "u16" => {
                            from_bytes.push(quote! {
                                #ident: {
                                    let mut b: [u8; 2] = [0; 2];
                                    if let Ok(len) = cur.read(&mut b) {
                                        if len != 2 {
                                            return Err(PacketParseError::NotEnoughBytes);
                                        }
                                    }
                                    else {
                                        return Err(PacketParseError::NotEnoughBytes);
                                    };
                                    u16::from_le_bytes(b)
                                },
                            });
                        },
                        "u32" => {
                            if ident_str == "flag" {
                                if i != 0 {
                                    return syn::Error::new(ident.span(), "flag must be first member of struct").to_compile_error().into();
                                }
                                has_flag = true;
                                continue;
                            }
                            from_bytes.push(quote! {
                                #ident: {
                                    let mut b: [u8; 4] = [0; 4];
                                    if let Ok(len) = cur.read(&mut b) {
                                        if len != 4 {
                                            return Err(PacketParseError::NotEnoughBytes);
                                        }
                                    }
                                    else {
                                        return Err(PacketParseError::NotEnoughBytes);
                                    };
                                    u32::from_le_bytes(b)
                                },
                            });
                        },
                        _ => {
                            return syn::Error::new(path.path.segments[0].ident.span(), "type not supported")
                                .to_compile_error().into();
                        }
                    }
                    partialeq.push(quote! {
                        if self.#ident != other.#ident {
                            return false;
                        }
                    });
                }
                _ => {
                }
            }
        }
    }

    let this_struct = parsed.ident.clone();
    let this_struct_str = this_struct.to_string();

    let flag_write = if has_flag {
        quote! {
            buf.extend_from_slice(&u32::to_le_bytes(self.flag));
        }
    }
    else {
        quote! {
            buf.extend_from_slice(&u32::to_le_bytes(0));
        }
    };

    let psopacket = quote! {
        impl PSOPacket for #this_struct {
            fn from_bytes(data: &Vec<u8>) -> Result<#this_struct, PacketParseError> {
                let mut cur = std::io::Cursor::new(data);
                cur.seek(SeekFrom::Start(2)).unwrap();
                let mut b: [u8; 2] = [0; 2];
                cur.read(&mut b).unwrap();
                let cmd = u16::from_le_bytes(b);

                if cmd != #pkt_cmd {
                    return Err(PacketParseError::WrongPacketCommand);
                }
                
                if #has_flag {
                    cur.seek(SeekFrom::Start(4)).unwrap();
                }
                else {
                    cur.seek(SeekFrom::Start(8)).unwrap();
                }
                Ok(#this_struct {
                    #(#from_bytes)*
                })
            }
            fn as_bytes(&self) -> Vec<u8> {
                let mut buf: Vec<u8> = Vec::new();
                #flag_write
                #(#as_bytes)*

                let pkt_len = buf.len() as u16;
                let mut prebuf: Vec<u8> = Vec::new();

                prebuf.extend_from_slice(&u16::to_le_bytes(pkt_len));
                prebuf.extend_from_slice(&u16::to_le_bytes(#pkt_cmd));
                prebuf.append(&mut buf);

                prebuf
            }
        }
    };
    
    
    let psopacket_debug = quote! {
        impl std::fmt::Debug for #this_struct {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "packet {} {{\n", #this_struct_str).unwrap();
                #(#dbg_write_vars)*
                write!(f, "}}")
            }
        }
    };


    let psopacket_partialeq = quote! {
        impl std::cmp::PartialEq for #this_struct {
            fn eq(&self, other: &Self) -> bool {
                #(#partialeq)*
                true
            }
        }
    };

    let q = quote! {
        #parsed
        #psopacket
        #psopacket_debug
        #psopacket_partialeq
    };

    //println!("[[[{}]]]", q.to_string());
    
    q.into()
}


#[proc_macro_attribute]
pub fn game_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
