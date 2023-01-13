use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ReadPacket)]
/// This derive macro will implement the `BanchoPacketRead` trait for the struct.
///
/// ### Usage
///
/// ```
/// use bancho_packets::{ReadPacket, PacketReader, PayloadReader};
///
/// #[derive(Debug, Clone, ReadPacket)]
/// /// [`BanchoMessage`] is the message structure of the bancho client.
/// pub struct BanchoMessage {
///     pub sender: String,
///     pub content: String,
///     pub target: String,
///     pub sender_id: i32,
/// }
///
/// // Now we can use [`PayloadReader`] to read the [`BanchoMessage`] from bytes.
/// let mut reader = PacketReader::new(&[
///     1, 0, 0, 20, 0, 0, 0, 11, 0, 11, 6, 228, 189, 160, 229, 165, 189,
///     11, 4, 35, 111, 115, 117, 0, 0, 0, 0,
/// ]);
/// let packet = reader.next().unwrap();
///
/// let mut payload_reader = PayloadReader::new(packet.payload.unwrap());
/// let message = payload_reader.read::<BanchoMessage>();
///
/// println!("{:?}: {:?}", packet.id, message);
/// ```
pub fn derive_read_packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        _ => panic!("#[derive(ReadPacket)] is only defined for structs."),
    };

    let field_names = fields.iter().map(|field| &field.ident);

    let expanded = quote! {
        impl BanchoPacketRead<#name> for #name {
            #[inline]
            fn read(reader: &mut PayloadReader) -> Option<#name> {
                Some(#name {
                    #(#field_names: reader.read()?,)*
                })
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(WritePacket)]
/// This derive macro will implement the `BanchoPacketWrite` trait for the struct.
///
/// ### Usage
///
/// ```
/// use bancho_packets::{ReadPacket, WritePacket, PacketLength};
///
/// #[derive(Debug, Clone, ReadPacket, WritePacket, PacketLength)]
/// /// The [`ScoreFrame`] uploaded by the bancho client during multiplayer games.
/// pub struct ScoreFrame {
///     pub timestamp: i32,
///     pub id: u8,
///     pub n300: u16,
///     pub n100: u16,
///     pub n50: u16,
///     pub geki: u16,
///     pub katu: u16,
///     pub miss: u16,
///     pub score: i32,
///     pub combo: u16,
///     pub max_combo: u16,
///     pub perfect: bool,
///     pub hp: u8,
///     pub tag_byte: u8,
///     pub score_v2: bool,
/// }
/// ```
pub fn derive_write_packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        _ => panic!("#[derive(WritePacket)] is only defined for structs."),
    };

    let field_names = fields.iter().map(|field| &field.ident);

    let expanded = quote! {
        impl BanchoPacketWrite for #name {
            #[inline]
            fn write_buf(self, buf: &mut Vec<u8>) {
                buf.extend(data!(
                    #(self.#field_names,)*
                ));
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(PacketLength, attributes(length))]
/// This derive macro will implement the `BanchoPacketLength` trait for the struct.
///
/// ### Usage
///
/// ```
/// use bancho_packets::{PacketLength};
///
/// #[derive(Debug, Clone, PacketLength)]
/// /// [`MatchData`] is the data of bancho client multiplayer game room.
/// pub struct MatchData {
///     pub match_id: i32,
///     pub in_progress: bool,
///     pub match_type: i8,
///     pub play_mods: u32,
///     pub match_name: String,
///     #[length(self.password.as_ref().map(|pw| pw.packet_len()).unwrap_or(2))]
///     pub password: Option<String>,
///     pub beatmap_name: String,
///     pub beatmap_id: i32,
///     pub beatmap_md5: String,
///     pub slot_status: Vec<u8>,
///     pub slot_teams: Vec<u8>,
///     pub slot_players: Vec<i32>,
///     pub host_player_id: i32,
///     pub match_game_mode: u8,
///     pub win_condition: u8,
///     pub team_type: u8,
///     pub freemods: bool,
///     pub player_mods: Vec<i32>,
///     pub match_seed: i32,
/// }
/// ```
pub fn derive_packet_length(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        _ => panic!("#[derive(PacketLength)] is only defined for structs."),
    };

    let mut extra = Vec::new();

    let field_names = fields
        .iter_mut()
        .filter(|field| {
            for attr in field.attrs.iter() {
                if attr.path.is_ident(&format_ident!("length")) {
                    extra.push(attr.tokens.clone());
                    return false;
                }
            }
            true
        })
        .enumerate()
        .map(|(i, field)| match &field.ident {
            Some(ident) => ident.clone(),
            None => Ident::new(&i.to_string(), Span::call_site()),
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl BanchoPacketLength for #name {
            #[inline]
            fn packet_len(&self) -> usize {
                let mut len = 0;
                #(len += self.#field_names.packet_len();)*
                #(len += #extra;)*
                len
            }
        }
    };

    expanded.into()
}
