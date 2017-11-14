# About

Rustix is a [matrix](https://matrix.org) bot/library/framework written in
[rust](https://www.rust-lang.org/). No matrix library is used; instead, this
project makes the http requests directly to a matrix server using the reqwest
library.

# Running

Rustix uses a database to keep quotes and track "karma" (e.g. rust++ or cabbage--)
to record likes and dislikes in a channel. To set up the database,
first, create a file called `.env` which contains a database url to a PostgreSLQ
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
through child nodes. This makes rustix very flexible. Examples: "self" filter
and prefix filter nodes are prebuilt and it is recommended that new services be
added under the prefix filter which is under the "self" filter. These filters only
propagate events to children processing nodes if the message wasn't sent by the
bot itself and the message starts with a prefix (which gets stripped off).

# Prebuilt commands
The framework should be fairly flexible and not too difficult to use for your own
project or just extend the existing. The following are prebuilt commands (and
should be prefixed with a `!`).

- addquote \<quote here\>
- getquote \<quote number\>
- roulette
- choose \<item1\> \<item2\> ... \<itemN\>
- echo \<string\>
- karma \<entity\>
- timecube

# Note

This is one of my first rust projects, so it would not be prudent to assume
that it is entirely idiomatic and a good reference to learning style from.
If you see something that could be improved (of which I'm sure there are many),
by all means, please open an issue and/or PR! I open to feedback, and I'd love to
make this better!
