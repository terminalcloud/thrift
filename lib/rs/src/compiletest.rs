#![allow(dead_code)]
use std::collections::{HashSet, HashMap};

strukt! {
    name = Simple,
    fields = {
        key: String => 16
    }
}

strukt! {
    name = DeeplyNested,
    fields = {
        nested: HashSet<Vec<Vec<Vec<Vec<i32>>>>> => 6
    }
}

strukt! {
    name = ReferencesOther,
    fields = {
        other: DeeplyNested => 2,
        another: Simple => 3,
        map: HashMap<i32, Vec<String>> => 4
    }
}

enom! {
    name = Operation,
    values = [Add = 1, Sub = 2, Mul = 3, Div = 4],
    default = Add
}

service! {
    trait_name = SharedService,
    processor_name = SharedServiceProcessor,
    client_name = SharedServiceClient,
    methods = [
        SharedServiceGetStructArgs -> SharedServiceGetStructResult = shared.get_struct(key: i32 => 1) -> DeeplyNested
    ],
    bounds = [<S: SharedService>],
    fields = [shared: S]
}

