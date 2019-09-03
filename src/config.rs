use confy;
use std::collections::HashMap;
use lazy_static::*;
use const_cstr::const_cstr;
use backend_glfw::imgui::*;
use palette;
use num_derive::FromPrimitive;
use log::*;
use enum_map::{enum_map, Enum, EnumMap};
use serde::{Serialize, Deserialize};

type Color = palette::rgb::Rgba;


// Named color choices
// based on Vicos 
// https://orv.banenor.no/orv/lib/exe/fetch.php?media=brukerveiledninger:symbolkatalog_vicos_-_iup-00-s-20385_00e_001.pdf
//
//
//  platform: turquoise
//
//  train routes
//    free tvd: gray
//    occupied tvd: red
//    reserved tvd: green
//    overlap tvd: orange
//  (add blink when releasing, if release has any delay (currently not modelled))
//
//  Shunting route:
//    reserved tvd: yellow
//    occupied tvd: turquoise
//  (add blink when releasing, if release has any delay (currently not modelled))
//
//  Operator has blocked route: two red cross-bars over track
//
//  switch:
//    remove track part which is not connected 
//        ---         /--
//     ------   vs. -/   ---- 
//    operator blocked switch: red box
//    switch locked/blocked (by route?): small yellow track section
//    providing flank protection: blue dot in fouling point
//
//  main signals
//    not used: gray triangle outline
//    part of train route: red triangle outline
//    proceed: green triangle outline
//    blocked by operator(?): red filled triangle
//    other: automatic etc...
//
//  shunting signal:
//    arrow instead of triangle
//
//
//


lazy_static! {
    pub static ref COLORNAMES :EnumMap<RailUIColorName, const_cstr::ConstCStr> = {
        enum_map! {
                RailUIColorName::CanvasBackground => const_cstr!("Canvas background"),
                RailUIColorName::CanvasGridPoint => const_cstr!("Canvas grid point"),
                RailUIColorName::CanvasSymbol => const_cstr!("Canvas symbol"),
                RailUIColorName::CanvasSymbolSelected => const_cstr!("Canvas symbol selected"),
                RailUIColorName::CanvasSymbolLocError => const_cstr!("Canvas symbol location error"),
                RailUIColorName::CanvasSignalStop => const_cstr!("Canvas signal stop"),
                RailUIColorName::CanvasSignalProceed => const_cstr!("Canvas signal proceed"),
                RailUIColorName::CanvasTrack => const_cstr!("Canvas track"),
                RailUIColorName::CanvasTrackDrawing => const_cstr!("Canvas drawing track"),
                RailUIColorName::CanvasTrackSelected => const_cstr!("Canvas track selected"),
                RailUIColorName::CanvasNode => const_cstr!("Canvas node"),
                RailUIColorName::CanvasNodeSelected => const_cstr!("Canvas node selected"),
                RailUIColorName::CanvasNodeError => const_cstr!("Canvas node error"),
                RailUIColorName::CanvasTrain => const_cstr!("Canvas train "),
                RailUIColorName::CanvasTrainSight => const_cstr!("Canvas train sighted signal"),
                RailUIColorName::CanvasTVDFree => const_cstr!("Canvas TVD free"),
                RailUIColorName::CanvasTVDOccupied => const_cstr!("Canvas TVD occupied"),
                RailUIColorName::CanvasTVDReserved => const_cstr!("Canvas TVD reserved"),
                RailUIColorName::CanvasRoutePath => const_cstr!("Canvas route path"),
                RailUIColorName::CanvasRouteSection => const_cstr!("Canvas route section"),
                RailUIColorName::CanvasSelectionWindow => const_cstr!("Canvas selection window"),
                RailUIColorName::GraphBackground => const_cstr!("Graph background"),
                RailUIColorName::GraphTimeSlider => const_cstr!("Graph time slider"),
                RailUIColorName::GraphTimeSliderText => const_cstr!("Graph time slider text"),
                RailUIColorName::GraphBlockBorder => const_cstr!("Graph block border"),
                RailUIColorName::GraphBlockReserved => const_cstr!("Graph block reserved"),
                RailUIColorName::GraphBlockOccupied => const_cstr!("Graph block occupied"),
                RailUIColorName::GraphTrainFront => const_cstr!("Graph train front"),
                RailUIColorName::GraphTrainRear => const_cstr!("Graph train rear"),
                RailUIColorName::GraphCommandRoute => const_cstr!("Graph command route"),
                RailUIColorName::GraphCommandTrain => const_cstr!("Graph command train"),
                RailUIColorName::GraphCommandError => const_cstr!("Graph command error"),
                RailUIColorName::GraphCommandBorder => const_cstr!("Graph command border"),
        }
    };
}

#[derive(Debug)]
pub struct Config {
    pub colors :EnumMap<RailUIColorName,Color>,
}


/// serde-friendly representation of the config struct
#[derive(Serialize,Deserialize)]
#[derive(Debug)]
pub struct ConfigString {
    pub colors :Vec<(String,String)>,  // name -> hex color
}

fn to_hex(c :Color) -> String {
    use palette::encoding::pixel::Pixel;
    let px  :[u8;4] = c.into_format().into_raw();
    format!("#{:02x}{:02x}{:02x}{:02x}", px[0],px[1],px[2],px[3])
}

fn from_hex(mut s :&str) -> Result<Color, ()> {
    // chop off '#' char
    if s.len() % 2 != 0 { s = &s[1..]; }
    if !(s.len() == 6 || s.len() == 8) { return Err(()); }
    // u8::from_str_radix(src: &str, radix: u32) converts a string
    // slice in a given base to u8
    let r: u8 = u8::from_str_radix(&s[0..2], 16).map_err(|_| ())?;
    let g: u8 = u8::from_str_radix(&s[2..4], 16).map_err(|_| ())?;
    let b: u8 = u8::from_str_radix(&s[4..6], 16).map_err(|_| ())?;
    let a = if s.len() == 8 {
        u8::from_str_radix(&s[6..8], 16).map_err(|_| ())?
    } else { 255u8 };

    Ok(Color::new(r as f32 / 255.0,
                  g as f32 / 255.0,
                  b as f32 / 255.0,
                  a as f32 / 255.0))
}

impl Default for ConfigString {
    fn default() -> Self {
        let c : Config = Default::default();
        c.to_config_string()
    }
}

impl Config {

    pub fn load() -> Self {
        let config_s : ConfigString = confy::load(env!("CARGO_PKG_NAME")).
            unwrap_or_else(|e| {
                error!("Could not load config file: {}", e);
                Default::default()
            });
        let config : Config = Config::from_config_string(&config_s);
        config
    }

    pub fn save(&self) {
        if let Err(e) = confy::store(env!("CARGO_PKG_NAME"), self.to_config_string()) {
            error!("Could not save config file: {}", e);
        }
    }


    pub fn to_config_string(&self) ->  ConfigString {
        let mut colors = Vec::new();
        unsafe {
            for (c,val) in self.colors.iter() {
                colors.push((std::str::from_utf8_unchecked(COLORNAMES[c].as_cstr().to_bytes()).to_string(), 
                             to_hex(*val)));
            }
        }

        ConfigString {
            colors: colors,
        }
    }

    pub fn from_config_string(cs :&ConfigString) -> Self {
        let mut colors = default_colors();
        for (name,col_hex) in cs.colors.iter() {
            for (col_choice, name_cstr) in COLORNAMES.iter() {
                unsafe {
                    if std::str::from_utf8_unchecked(name_cstr.as_cstr().to_bytes()) == name {
                        if let Ok(c) = from_hex(col_hex) {
                            colors[col_choice] = c;
                        }
                    }
                }
            }
        }

        Config {
            colors: colors,
        }
    }

    pub fn get_font_size(&self) -> f32 { 16.0 }
    pub fn get_font_filename(&self) -> Option<String> {
        use font_kit::source::SystemSource;
        use font_kit::family_name::FamilyName;
        use font_kit::properties::Properties;
        use font_kit::handle::Handle;
        let font = SystemSource::new().select_best_match(&[
                                                 //FamilyName::Title("Segoe UI".to_string()),
                                                 FamilyName::SansSerif],
                                                 &Properties::new()).ok()?;
        match font {
            Handle::Path { path, font_index } => {
                info!("Using font {:?}", path);
                let f = path.to_string_lossy().to_string();
                Some(f)
            },
            _ => { None }
        }

    }



    pub fn color_u32(&self, name :RailUIColorName) -> u32 {
        let c = self.colors[name];
        unsafe { igGetColorU32Vec4(ImVec4 { x: c.color.red,  y: c.color.green, 
            z: c.color.blue, w: c.alpha  }) }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            colors: default_colors(),
        }
    }
}

pub fn default_colors() -> EnumMap<RailUIColorName, Color> {
    use palette::named;
    let c = |nm :palette::Srgb<u8>| {
        let f :palette::Srgb<f32> = palette::Srgb::from_format(nm);
        let a :Color = f.into();
        a
    };
    enum_map! {
        RailUIColorName::CanvasBackground => c(named::CORNSILK),
        RailUIColorName::CanvasGridPoint => c(named::BLANCHEDALMOND),
        RailUIColorName::CanvasSymbol => c(named::INDIGO),
        RailUIColorName::CanvasSymbolSelected => c(named::NAVY),
        RailUIColorName::CanvasSymbolLocError => c(named::ORANGERED),
        RailUIColorName::CanvasSignalStop => c(named::RED),
        RailUIColorName::CanvasSignalProceed => c(named::LIME),
        RailUIColorName::CanvasTrack => c(named::DARKSLATEBLUE),
        RailUIColorName::CanvasTrackDrawing => c(named::GOLDENROD),
        RailUIColorName::CanvasTrackSelected => c(named::NAVY),
        RailUIColorName::CanvasNode => c(named::BLACK),
        RailUIColorName::CanvasNodeSelected => c(named::NAVY),
        RailUIColorName::CanvasNodeError => c(named::RED),
        RailUIColorName::CanvasTrain => c(named::TOMATO),
        RailUIColorName::CanvasTrainSight => c(named::ORANGE),
        RailUIColorName::CanvasTVDFree => c(named::BLACK),
        RailUIColorName::CanvasTVDOccupied => c(named::SALMON),
        RailUIColorName::CanvasTVDReserved => c(named::SLATEBLUE),
        RailUIColorName::CanvasRoutePath => c(named::DARKSLATEBLUE),
        RailUIColorName::CanvasRouteSection => c(named::SLATEBLUE),
        RailUIColorName::CanvasSelectionWindow => c(named::NAVY),
        RailUIColorName::GraphBackground => c(named::HONEYDEW),
        RailUIColorName::GraphTimeSlider => c(named::LIGHTSALMON),
        RailUIColorName::GraphTimeSliderText => c(named::DARKGREY),
        RailUIColorName::GraphBlockBorder => c(named::IVORY),
        RailUIColorName::GraphBlockReserved => c(named::LIGHTSKYBLUE),
        RailUIColorName::GraphBlockOccupied => c(named::LIGHTPINK),
        RailUIColorName::GraphTrainFront => c(named::TOMATO),
        RailUIColorName::GraphTrainRear => c(named::TOMATO),
        RailUIColorName::GraphCommandRoute => c(named::LIMEGREEN),
        RailUIColorName::GraphCommandTrain => c(named::AZURE),
        RailUIColorName::GraphCommandError => c(named::RED),
        RailUIColorName::GraphCommandBorder => c(named::BLACK),
    }
}

#[derive(Enum, FromPrimitive, Debug, PartialEq, Eq, Copy, Clone)]
#[derive(Serialize,Deserialize)]
pub enum RailUIColorName {
    CanvasBackground,
    CanvasGridPoint,
    CanvasSymbol,
    CanvasSymbolSelected,
    CanvasSymbolLocError,
    CanvasSignalStop,
    CanvasSignalProceed,
    CanvasTrack,
    CanvasTrackDrawing,
    CanvasTrackSelected,
    CanvasNode,
    CanvasNodeSelected,
    CanvasNodeError,
    CanvasTrain,
    CanvasTrainSight,
    CanvasTVDFree,
    CanvasTVDOccupied,
    CanvasTVDReserved,
    CanvasRoutePath,
    CanvasRouteSection,
    CanvasSelectionWindow,
    GraphBackground,
    GraphTimeSlider,
    GraphTimeSliderText,
    GraphBlockBorder,
    GraphBlockReserved,
    GraphBlockOccupied,
    GraphTrainFront,
    GraphTrainRear,
    GraphCommandRoute,
    GraphCommandTrain,
    GraphCommandError,
    GraphCommandBorder,
}

#[test]
pub fn colr_no() {
    use num_traits::FromPrimitive;
    let x = RailUIColor::from_usize(2);
    dbg!(x.unwrap());
    assert_eq!(x.unwrap(), RailUIColor::TVDReserved);
}


