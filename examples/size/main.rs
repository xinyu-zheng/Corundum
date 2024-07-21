use derive_fields::derivefields;
use corundum::default::*;
use corundum::open_flags::*;
use std::env;

// i64: 8 * 32 KB
#[derivefields(i64, "f", 32768)]
#[derive(Default)]
pub struct Node {
    f0: i64,
}



fn main() {
    let args: Vec<std::string::String> = env::args().collect();
    type P = Allocator;
    struct Root { val: Pbox<PRefCell<Node>> }
    impl RootObj<P> for Root {
        fn init(j: &Journal) -> Self { Self{
            val: Pbox::new(PRefCell::new(Default::default()), j)
        }}
    }
    let root = P::open::<Root>(&args[1], O_CFNE | O_1GB).unwrap();
    for i in 0..1000 {
        P::transaction(|j| {
            root.val.borrow_mut(j).f32000 = i;
            
        }).unwrap();
    }
}