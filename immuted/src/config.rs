use lazy_static::*;
use const_cstr::const_cstr;
use backend_glfw::imgui::*;
use palette;
use num_derive::FromPrimitive;
use enum_map::{enum_map, Enum, EnumMap};

type Color = palette::rgb::Rgba;

lazy_static! {
    static ref COLORNAMES :EnumMap<RailUIColorName, const_cstr::ConstCStr> = {
        enum_map! {
                RailUIColorName::CanvasBackground => const_cstr!("Canvas background"),
                RailUIColorName::CanvasGridPoint => const_cstr!("Canvas grid point"),
                RailUIColorName::CanvasSymbol => const_cstr!("Canvas symbol"),
                RailUIColorName::CanvasSymbolSelected => const_cstr!("Canvas symbol selected"),
                RailUIColorName::CanvasSymbolLocError => const_cstr!("Canvas symbol location error"),
                RailUIColorName::CanvasTrack => const_cstr!("Canvas track"),
                RailUIColorName::CanvasTrackDrawing => const_cstr!("Canvas drawing track"),
                RailUIColorName::CanvasTrackSelected => const_cstr!("Canvas track selected"),
                RailUIColorName::CanvasNode => const_cstr!("Canvas node"),
                RailUIColorName::CanvasNodeSelected => const_cstr!("Canvas node selected"),
                RailUIColorName::CanvasTrain => const_cstr!("Canvas train "),
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
                RailUIColorName::GraphCommand => const_cstr!("Graph command"),
        }
    };
}

pub struct Config {
    pub colors :EnumMap<RailUIColorName,Color>,
}

impl Config {
    pub fn default() -> Config {
        use palette::named;
        let c = |nm :palette::Srgb<u8>| {
            let f :palette::Srgb<f32> = palette::Srgb::from_format(nm);
            let a :Color = f.into();
            a
        };
        Config {
            colors: enum_map! {
                RailUIColorName::CanvasBackground => c(named::CORNSILK),
                RailUIColorName::CanvasGridPoint => c(named::BLANCHEDALMOND),
                RailUIColorName::CanvasSymbol => c(named::INDIGO),
                RailUIColorName::CanvasSymbolSelected => c(named::NAVY),
                RailUIColorName::CanvasSymbolLocError => c(named::ORANGERED),
                RailUIColorName::CanvasTrack => c(named::DARKSLATEBLUE),
                RailUIColorName::CanvasTrackDrawing => c(named::GOLDENROD),
                RailUIColorName::CanvasTrackSelected => c(named::NAVY),
                RailUIColorName::CanvasNode => c(named::BLACK),
                RailUIColorName::CanvasNodeSelected => c(named::NAVY),
                RailUIColorName::CanvasTrain => c(named::TOMATO),
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
                RailUIColorName::GraphCommand => c(named::LIMEGREEN),
            },
        }
    }

    pub fn color_u32(&self, name :RailUIColorName) -> u32 {
        let c = self.colors[name];
        unsafe { igGetColorU32Vec4(ImVec4 { x: c.color.red,  y: c.color.green, 
            z: c.color.blue, w: c.alpha  }) }
    }
}

#[derive(Enum, FromPrimitive, Debug, PartialEq, Eq, Copy, Clone)]
pub enum RailUIColorName {
    CanvasBackground,
    CanvasGridPoint,
    CanvasSymbol,
    CanvasSymbolSelected,
    CanvasSymbolLocError,
    CanvasTrack,
    CanvasTrackDrawing,
    CanvasTrackSelected,
    CanvasNode,
    CanvasNodeSelected,
    CanvasTrain,
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
    GraphCommand,
}

pub fn edit_config_window(popen :&mut bool, config :&mut Config) {
    unsafe {
    igBegin(const_cstr!("Configuration").as_ptr(), popen as _, 0 as _);
    edit_config(config);
    igEnd();
    }

}

pub fn edit_config(config :&mut Config) {
    unsafe {
        for (name,color) in config.colors.iter_mut() {
            let name = COLORNAMES[name].as_ptr();
            igColorEdit4(name, &mut color.color.red as _, 0 as _);
        }
    }
}


#[test]
pub fn colr_no() {
    use num_traits::FromPrimitive;
    let x = RailUIColor::from_usize(2);
    dbg!(x.unwrap());
    assert_eq!(x.unwrap(), RailUIColor::TVDReserved);
}


