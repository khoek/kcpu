use kcpu::assembler::{
    self, model,
    phases::{
        generate,
        types::{Loc, Located},
    },
    Error,
};

#[test]
fn at_most_one_const_in_inst() {
    assert_eq!(
        assembler::assemble("ST $0x500 $0x500"),
        Err::<Vec<u16>, _>(Error::Generate(Located::with_loc(
            Loc::new(1, 1),
            generate::Error::InstMultipleConstArgs(
                String::from("ST"),
                vec![
                    model::Arg::Const(model::ConstBinding::Resolved(model::Const::Word(0x500))),
                    model::Arg::Const(model::ConstBinding::Resolved(model::Const::Word(0x500)))
                ]
            )
        )))
    );
}

#[test]
fn more_than_one_const_in_alias() {
    // RUSTFIX IMPLEMENT
    // todo!();
}
