use syn::{Data, DataEnum, DataStruct, DataUnion, Fields};
use Data::{Enum, Struct, Union};

#[inline(always)]
pub(crate) fn get_struct_fields(input: syn::DataStruct) -> Fields {
    input.fields
}

pub(crate) fn get_data_type<T>(input: Data) -> T
where
    T: From<DataStruct> + From<DataEnum> + From<DataUnion>,
{
    match input {
        Struct(data) => data.into(),
        Enum(data) => data.into(),
        Union(data) => data.into(),
    }
}
