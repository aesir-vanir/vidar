//! `vidar` errors

error_chain!{
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        InvalidKind(name: String) {
            description("Invalid kind name supplied!")
            display("Invalid kind {} supplied!", name)
        }
        InvalidProperty {
            description("Invalid property found!")
            display("Invalid property found!")
        }
    }
}
