trait BeanCounter {} // WARN: NO DOCS

type Deaf = String; //WARN: NO DOCS

/// A warm welcome, but not useful if you call it on Deaf. //WARN: unlinked ident.
struct Hi {
    h: String,
}

/// Much more useful than [`Hi`] //INFO: NO PROBLEM
enum Hello {
    One,
    Two,
}
// our program begins here
fn main() {
    println!("Hello, world!");
}
