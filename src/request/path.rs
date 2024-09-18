use crate::Str;

pub struct Path(Str);

const _/* trait impls */: () = {
    impl std::fmt::Debug for Path {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }
};
