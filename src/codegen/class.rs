use model::ir; // will use ast, too

// will take more arguments, probably
pub fn get_size_of(type_: &ir::Type) -> i32 {
    use self::ir::Type::*;
    match type_ {
        Void => unreachable!(),
        Int => 4,
        Bool => 1,
        Char => 1,
        Ptr(_) => 8, // 64-bit
        Struct(_) => unimplemented!(),
    }
}
