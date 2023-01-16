use super::bot::Nekobot;
use super::map::NystopiaMap;

pub trait Renderer {
    fn new() -> Self;
    fn init(self: &mut Self) -> Result<(), Box<dyn std::error::Error>>;
    fn blank(self: &mut Self) -> Result<(), Box<dyn std::error::Error>>;
    fn render_map(self: &mut Self, map: &NystopiaMap) -> Result<(), Box<dyn std::error::Error>>;
    fn get_rows(self: &Self) -> Result<u16, Box<dyn std::error::Error>>;
    fn get_cols(self: &Self) -> Result<u16, Box<dyn std::error::Error>>;
    fn place_tile(
        self: &mut Self,
        map: &NystopiaMap,
        row: u16,
        col: u16,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn place_bot(
        self: &mut Self,
        map: &NystopiaMap,
        bot: &Nekobot,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
