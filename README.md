# About

Rustix is a [matrix](https://matrix.org) bot/library/framework written in
[rust](https://www.rust-lang.org/). This project does not use a matrix client
library, but rather contains one within it! HTTP requests are made directly to a
matrix server via the reqwest library.

# Running

To run rustix there must be a matrix user account with a password set up. The
username and password should be put in `config.toml`. Rustix uses a database to
keep quotes and track "karma" (e.g. rust++ or cabbage--) to record likes and
dislikes in a channel. To set up the database, first, create a file called
`.env` which contains a database url to a PostgreSQL database. It should look
something like this:
```
DATABASE_URL=postgres://user:password@localhost/rustix
```
(Note: This assumes a database user has been set up and has proper permissions
on the proper database.)

Next, run the database migrations and comple+run rustix!
```
$ diesel migration run
$ cargo run
```

# Architecture

The command/plugin/service architecture of rustix can be thought of as a
directed graph. Service nodes are added to the graph and matrix events get
propogated (or blocked) through child nodes. This makes rustix very flexible.
Examples: "self" filter and prefix filter nodes are prebuilt and it is
recommended that new services be added under the prefix filter which is under
the "self" filter. These filters only propagate events to child processing nodes
if the message wasn't sent by the bot itself and the message starts with a
prefix (which gets stripped off).

# Prebuilt commands
The framework should be fairly flexible and not too difficult to use for your
own project or to just extend. The following are prebuilt commands, and should
be prefixed with the default prefix: `!`. (The prefix can be changed in
`config.toml`)

- addquote \<quote here\>
- getquote \<quote number\>
- \*delquote \<quote number\>
- randquote \<optional string search\>
- searchquote \<string to search\>
- oldgetquote \<quote number\>
- oldrandquote \<optional string search\>
- oldsearchquote \<string search\>
- roulette
- rroulette
- choose \<item1\> \<item2\> ... \<itemN\>
- echo \<string\>
- karma \<entity\>
- p \<crypto currency ticker\>
- \*join \<public channel display name\>
- \*leave \<public channel display name\>
- \*joined
- help

\**Command is under the admin node and requires message sender to be in the
admin list specified in `config.toml`*

There is also TryFile, which attempts to look for file matching
`!<command name>` to a file `<command name>.txt` in the `var` folder in the
current working directory and echo a random line from it. This allows for
invocation like `!timecube` which will echo a random line from
`var/timecube.txt`.  N.B. If there is a command name collision both matched
commands will trigger, that is if one were to place a file named `randquote.txt`
in `var`, both the randquote function will be executed and a random line from
`randquote.txt` will be echoed.

# Config

Rustix expects a file named `config.toml` to be in the current working
directory. This file should look something like this:

```
[connection]
server = "https://matrix.my.domain.com/"
username = "rustix"
password = "mySecr3tPassword"

[bot]
display_name = "rustix"
prefix = "!"
rooms = ["general", "rust", "memes"]
admins = ["@myself:matrix.my.domain.com"]
ignore = ["@bot1:matrix.my.domain.com", "@bot2:matrix.my.domain.com"]

[services]
[services.try_file]
directory = "var"
```

Rustix will ignore all events by users in the ignore list, not just ignore
commands.

# Docker

There is a Makefile with phony targets which makes running a dockerized version
of rustix a breeze. Before running this way, note the instructions make the
following assumptions:

1. Make is installed
2. `config.toml` has been appropriately configured
3. `services.try_file.directory` in `config.toml` is set to
   `"/usr/share/rustix"`
4. Docker and docker-compose are both installed and setup
5. Any files to be used with the tryfile service are in a folder named `var` in
   the project root

### Step 1

Run `make rustix` which builds the main rustix image

### Step 2

Run `make migration` which builds the db maintenence (for db migrations) image

### Step 3

Run `make setup` which generates a database password, launches rustix and the db
maintenence container (which then runs the db migrations), copies files from `var`
in to a volume, and removes the db maintenence container after running
migrations.

Rustix should now be running. From here you can easily run `make up`, `make
down`, `make stop` and `make start` which are simple helpful wrappers around the
respective docker-compose commands.

*NOTE:* Upon running this command a file named `.pw_lock` will be created which
contains the password to the postgres database which rustix uses. This file
could alternately be created before running `make setup` and set to whatever you
like, or removed and thus ephemeral.  Keeping it around allows easy
stopping/starting of the rustix container independent of any other helper
containers (such as the postgres container).

# Note

If you see something that could be improved (I'm sure there are many things), by
all means, please open an issue and/or PR! I'm open to feedback, and I would
love to improve this project!
