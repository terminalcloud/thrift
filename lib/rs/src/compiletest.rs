#![allow(dead_code)]
use std::collections::{HashSet, HashMap};

strukt! {
    name = Simple,
    fields = {
        key: String => 16,
    }
}

strukt! {
    name = DeeplyNested,
    fields = {
        nested: HashSet<Vec<Vec<Vec<Vec<i32>>>>> => 6,
    }
}

strukt! {
    name = ReferencesOther,
    fields = {
        other: DeeplyNested => 2,
        another: Simple => 3,
        map: HashMap<i32, Vec<String>> => 4,
    }
}

enom! {
    name = Operation,
    values = [Add = 1, Sub = 2, Mul = 3, Div = 4,],
    default = Add
}

service! {
    trait_name = SharedService,
    processor_name = SharedServiceProcessor,
    client_name = SharedServiceClient,
    service_methods = [
        SharedServiceGetStructArgs -> SharedServiceGetStructResult = shared.get_struct(key: i32 => 1,) -> DeeplyNested => [],
    ],
    parent_methods = [],
    bounds = [S: SharedService,],
    fields = [shared: S,]
}

service! {
     trait_name = ChildService,
     processor_name = ChildServiceProcessor,
     client_name = ChildServiceClient,
     service_methods = [
         ChildServiceOperationArgs -> ChildServiceOperationResult = child.operation(
             one: String => 2,
             another: i32 => 3,
         ) -> Operation => [],
     ],
     parent_methods = [
        SharedServiceGetStructArgs -> SharedServiceGetStructResult = shared.get_struct(key: i32 => 1,) -> DeeplyNested => [],
     ],
     bounds = [S: SharedService, C: ChildService,],
     fields = [shared: S, child: C,]
}

strukt! {
     name = Exception,
     fields = {
          name: String => 0,
          message: String => 1,
     }
}

service! {
    trait_name = ServiceWithException,
    processor_name = ServiceWithExceptionProcessor,
    client_name = ServiceWithExceptionClient,
    service_methods = [
        ServiceWithExceptionOperationArgs -> ServiceWithExceptionOperationResult = this.operation() -> i32 => [bad: Exception => 1,],
    ],
    parent_methods = [],
    bounds = [S: ServiceWithException,],
    fields = [this: S,]
}

