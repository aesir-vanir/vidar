//! `vidar` errors

error_chain!{
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        ConfigPath {
            description("Unable to determine the configuration path for your application!")
            display("Unable to determine the configuration path for your application!")
        }
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
