# About

Rustix is a matrix bot written in rust. No matrix library is used, this simply
makes the http requests to a matrix server using the reqwest library.

# Running

First, you must create a file called `.env` which contains a database url to a PostgreSLQ database. It should look something like this:
```
DATABASE_URL=user:password//localhost/rustix
```

Next, run the database migrations and comple+run rustix!
```
$ diesel migration run
$ cargo run
```

# Architecture

The architecture of rustix is one similar to a graph. Nodes are added to the
graph and matrix events get propogated (or blocked) down through child nodes.


# Note

This is one of my first rust projects, so it would not be best to assume it is
entirely idiomatic and a good reference for learning style from. If you see some
things that could be improved, by all means please open an issue or PR! I'd love
to make this better.
