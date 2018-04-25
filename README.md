# About

Rustix is a [matrix](https://matrix.org) bot/library/framework written in
[rust](https://www.rust-lang.org/). No matrix library is used; this project
makes http requests directly to a matrix server via the reqwest library.

# Running

To run rustix there must be a matrix user account with a password set up. The
username and password should be put in `config.toml`. Rustix uses a database to
keep quotes and track "karma" (e.g. rust++ or cabbage--) to record likes and
dislikes in a channel. To set up the database, first, create a file called
`.env` which contains a database url to a PostgreSQL database. It should look
something like this:
```
DATABASE_URL=user:password//localhost/rustix
```
(Note: This assumes a database user has been set up and has proper permissions
on the proper database.)

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
added under the prefix filter which is under the "self" filter. These filters
only propagate events to child processing nodes if the message wasn't sent by
the bot itself and the message starts with a prefix (which gets stripped off).

# Prebuilt commands
The framework should be fairly flexible and not too difficult to use for your
own project or to just extend. The following are prebuilt commands, and should
be prefixed with the default prefix: `!`.

- addquote \<quote here\>
- getquote \<quote number\>
- randquote
- roulette
- choose \<item1\> \<item2\> ... \<itemN\>
- echo \<string\>
- karma \<entity\>
- p \<crypto currency ticker\>
- \*join \<public channel display name\>
- \*leave \<public channel display name\>

\**Command is under the admin node and requires message sender to be in the
admin list given in `config.toml`*

There is also TryFile, which attempts to look for file named `___.txt` in the
`var` folder in the current working directory and echo a random line from it.
This allows for things like `!timecube` which will echo a random line from
`var/timecube.txt`. If there is a name collision both things will happen, that
is if one were to place a file named `randquote.txt` in `var`, both the
randquote function will be executed and a random line from `randquote.txt` will
be echoed.

# Note

This is one of my first rust projects, thus it would not be prudent to assume
that it is entirely idiomatic and a good reference to learn style from.
If you see something that could be improved (I'm sure there are many things that
could be), by all means, please open an issue and/or PR! I'm open to feedback,
and I would love to improve this project!
