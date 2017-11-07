# About

Rustix is a matrix bot written in rust. No matrix library is used, this simply
makes the http requests to a matrix server using the reqwest library.

# Running

Rustix uses a database to keep track of "karma" (e.g. rust++ or cabbage--) to
record likes and dislikes in a channel. To set up the database, first, create a
file called `.env` which contains a database url to a PostgreSLQ
database. It should look something like this:
```
DATABASE_URL=user:password//localhost/rustix
```

Next, run the database migrations and comple+run rustix!
```
$ diesel migration run
$ cargo run
```

# Architecture

The command/plugin/service architecture of rustix is that of a graph. Service
nodes are added to the graph and matrix events get propogated (or blocked)
through child nodes. This makes rustix very flexible.


# Note

This is one of my first rust projects, so it would not be best to assume it is
entirely idiomatic and a good reference to learning style from. If you see
something that could be improved (of which I'm sure there are many), by all
means, please open an issue or PR! I don't bite, and I'd love to make this
better!
