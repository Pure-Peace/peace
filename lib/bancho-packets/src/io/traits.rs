pub mod reading {
    use crate::{BanchoMessage, PayloadReader, ScoreFrame};

    #[macro_export]
    macro_rules! read_struct {
        ($reader:ident, $struct:ident { $($field:ident),+ }) => {
            $struct {
                $($field: $reader.read()?,)+
            }
        };
    }

    pub trait OsuRead<T> {
        fn read(reader: &mut PayloadReader) -> Option<T>;
    }

    impl OsuRead<String> for String {
        fn read(reader: &mut PayloadReader) -> Option<String> {
            if reader.payload.get(reader.index())? != &0xb {
                return None;
            }
            reader.add_index(1);
            let data_length = reader.read_uleb128()? as usize;

            let cur = reader.index;
            reader.add_index(data_length);
            let data = reader.payload.get(cur..reader.index)?;

            Some(std::str::from_utf8(data).ok()?.into())
        }
    }

    impl OsuRead<bool> for bool {
        fn read(reader: &mut PayloadReader) -> Option<bool> {
            Some(reader.read::<i8>()? == 1)
        }
    }

    macro_rules! impl_number {
        ($($t:ty),+) => {
            $(impl OsuRead<$t> for $t {
                fn read(reader: &mut PayloadReader) -> Option<$t> {
                    Some(<$t>::from_le_bytes(
                        reader.next(std::mem::size_of::<$t>())?.try_into().ok()?,
                    ))
                }
            })+
        };
    }

    impl_number!(i8, u8, i16, u16, i32, u32, i64, u64);

    impl OsuRead<BanchoMessage> for BanchoMessage {
        fn read(reader: &mut PayloadReader) -> Option<BanchoMessage> {
            Some(read_struct!(
                reader,
                BanchoMessage {
                    sender,
                    content,
                    target,
                    sender_id
                }
            ))
        }
    }

    impl OsuRead<ScoreFrame> for ScoreFrame {
        fn read(reader: &mut PayloadReader) -> Option<ScoreFrame> {
            Some(read_struct!(
                reader,
                ScoreFrame {
                    timestamp,
                    id,
                    n300,
                    n100,
                    n50,
                    geki,
                    katu,
                    miss,
                    score,
                    combo,
                    max_combo,
                    perfect,
                    hp,
                    tag_byte,
                    score_v2
                }
            ))
        }
    }

    macro_rules! impl_number_array {
        ($($t:ty),+) => {
            $(impl OsuRead<Vec<$t>> for Vec<$t> {
                fn read(reader: &mut PayloadReader) -> Option<Vec<$t>> {
                    let length_data = reader.next(std::mem::size_of::<i16>())?;
                    let int_count = <i16>::from_le_bytes(length_data.try_into().ok()?) as usize;

                    let mut data = Vec::with_capacity(int_count);
                    for _ in 0..int_count {
                        data.push(<$t>::from_le_bytes(reader.next(4)?.try_into().ok()?));
                    }
                    Some(data)
                }
            })+
        };
    }

    impl_number_array!(i8, u8, i16, u16, i32, u32, i64, u64);
}

pub mod writing {
    use crate::{data, write_uleb128, MatchData, MatchDataSerialization, ScoreFrame};

    pub trait OsuWrite {
        fn osu_write(&self, buf: &mut Vec<u8>);
    }

    macro_rules! impl_number {
        ($($t:ty),+) => {
            $(impl OsuWrite for $t {
                #[inline]
                fn osu_write(&self, buf: &mut Vec<u8>) {
                    buf.extend(self.to_le_bytes())
                }
            })+
        }
    }

    macro_rules! impl_number_array {
        ($($t:ty),+) => {$(impl OsuWrite for [$t] {
            #[inline]
            fn osu_write(&self, buf: &mut Vec<u8>) {
                buf.extend((self.len() as u16).to_le_bytes());
                for int in self.iter() {
                    buf.extend(int.to_le_bytes())
                }
            }
        })+}
    }

    impl OsuWrite for str {
        #[inline]
        fn osu_write(&self, buf: &mut Vec<u8>) {
            let byte_length = self.len();
            if byte_length > 0 {
                buf.push(0xb);
                buf.extend(write_uleb128(byte_length as u32));
                buf.extend(self.as_bytes());
            } else {
                buf.push(0);
            }
        }
    }

    impl OsuWrite for u8 {
        #[inline]
        fn osu_write(&self, buf: &mut Vec<u8>) {
            buf.push(*self);
        }
    }

    impl OsuWrite for [u8] {
        #[inline]
        fn osu_write(&self, buf: &mut Vec<u8>) {
            buf.extend(self);
        }
    }

    impl OsuWrite for bool {
        #[inline]
        fn osu_write(&self, buf: &mut Vec<u8>) {
            buf.push(if *self { 1 } else { 0 });
        }
    }

    impl_number!(i8, u16, i16, i32, u32, i64, u64, f32, f64);
    impl_number_array!(i8, u16, i16, i32, u32, i64, u64, f32, f64);

    impl<'a> OsuWrite for MatchDataSerialization<'a> {
        fn osu_write(&self, buf: &mut Vec<u8>) {
            let raw_password = if let Some(pw) = self.password {
                if self.1 {
                    let mut buf = Vec::new();
                    pw.osu_write(&mut buf);
                    buf
                } else {
                    b"\x0b\x00".to_vec()
                }
            } else {
                b"\x00".to_vec()
            };

            buf.extend(data!(
                self.match_id as u16,
                self.in_progress,
                self.match_type,
                self.play_mods,
                self.match_name,
                raw_password,
                self.beatmap_name,
                self.beatmap_id,
                self.beatmap_md5,
                self.slot_status,
                self.slot_teams,
                self.slot_players,
                self.host_player_id,
                self.match_game_mode,
                self.win_condition,
                self.team_type,
                self.freemods,
                self.player_mods,
                self.match_seed
            ));
        }
    }

    impl<'a> OsuWrite for MatchData<'a> {
        fn osu_write(&self, buf: &mut Vec<u8>) {
            MatchDataSerialization(self, true).osu_write(buf);
        }
    }

    impl OsuWrite for ScoreFrame {
        fn osu_write(&self, buf: &mut Vec<u8>) {
            buf.extend(data!(
                self.timestamp,
                self.id,
                self.n300,
                self.n100,
                self.n50,
                self.geki,
                self.katu,
                self.miss,
                self.score,
                self.combo,
                self.max_combo,
                self.perfect,
                self.hp,
                self.tag_byte,
                self.score_v2
            ));
        }
    }
}
