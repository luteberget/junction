/// Custom data set for interlocking specification calcs.
pub struct DataSet {
}

type EntityId = ();
pub enum RecognizedStructure {
    Route {
        start: EntityId,
        end: EntityId,
        // ...
    },
    TrackInterval {
    },
    Area {
        delimiters :Vec<EntityId>,
    },
    // ...
}

pub type Json = ();

/// Try to recognize a JSON object as one of the recognized structures.
pub fn try_interpret(obj :&Json) -> Option<RecognizedStructure> {
    unimplemented!()
}

