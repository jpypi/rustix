# About

Rustix is a [matrix](https://matrix.org) bot/library/framework written in
[rust](https://www.rust-lang.org/). This project does not use a matrix client
library, but rather contains one, with only the necessary API calls, within it.
HTTP requests are made directly to a matrix server via the reqwest library.

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

Next, run the database migrations and comple + run rustix!
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
- roulette
- rroulette
- choose \<item1\> \<item2\> ... \<itemN\>
- echo \<string\>
- karma \<entity\>
- karmastats \<optional entity\>
- badkarmastats
- nickstats \<optional matrix user id\>
- badnickstats \<optional matrix user id\>
- p \<crypto currency ticker\>
- \*join \<public channel display name\>
- \*leave \<public channel display name\>
- \*joined
- help

\**Command is under the admin node and requires message sender to be in the
admin list specified in `config.toml`*

## Optional Commands (if configured)

### Enabled via `services.csv_quote`:
These commands will look up quotes stored in a csv file. New quotes must be
manually added to this file.

The expected column layout is: `id,text,user,timestamp,channel`
- oldgetquote \<quote number\>
- oldrandquote \<optional string search\>
- oldsearchquote \<string search\>

### Enabled via `services.try_file`:
The TryFile service attempts to interpret `!<command name>` as referring to a
file named `<command name>.txt` in the `var` folder in the current working
directory and then echo a random line from it. This allows for invocation like
`!timecube` which will echo a random line from `var/timecube.txt`.  N.B. If
there is a command name collision both matched commands will trigger, that is if
one were to place a file named `randquote.txt` in `var`, both the randquote
function will be executed and a random line from `randquote.txt` will be echoed.

### Enabled via `services.web_search`:
The `s` command, which performs search queries using the Goolge custom web
search API.

- s \<search string\>

### Enabled via `services.openai`:
The `chat` command, which enables interaction with openai's gpt models. Very
similar to having your own ChatGPT that everyone can interact with in a shared
channel.

- chat \<whatever you want to say to rustix\>


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
directory = "/usr/share/rustix"

[services.csv_quote]
file = "csv_quotes.csv"

[services.web_search]
key = "<google api key>"
seid = "<google custom search id>"

[services.openai]
secret = "<openai api key>"
backstory_file = "backstory.txt"
monthly_budget = 5.0
# starting_tokens is optional
starting_tokens = 10000
```

Rustix will ignore all events by users in the ignore list, not just commands.

The configuration for the following services is optional. That is, removing the
configuration will disable the service in rustix and not cause an error.

- try_file
- csv_quote
- web_search
- openai

# Docker - Pre-built (recommended/easiest)

There are pre-built rustix docker images in this gitlab project which the
`docker-compose.yml` file utilizes.  By default, docker compose will use a bind
mount for the `config.toml` file, but you could alternately copy the config in
to the rustix container via: `docker cp config.toml
rustix-rustix-1:config.toml` or utilize a volume.

You will also need to make sure to have the following in place:

1. Make is installed
2. Docker and docker-compose are both installed and setup
3. An appropriately configured `config.toml` file
4. A folder named `var`, containing all the files that the tryfile service can
   use, in the project root folder
5. If you intend to use the `old*quote` commands, a file named `csv_quotes.csv`
   lives in the project root folder

Run `Step 3` from the "Docker - DIY" section of this README (below).

# Docker - DIY

There is a Makefile which makes building and running a dockerized version of
rustix a breeze. Before running this way, note that the instructions make the
following assumptions:

1. Make is installed
2. Docker and docker-compose are both installed and setup
3. You have updated `docker-compose.yml` to use the appropriate local images:
   `perplexinglabs/rustix:0.1` and `perplexinglabs/rustix-diesel:0.1`
   (as defined in the `Makefile`)
4. `config.toml` has been appropriately configured
5. A folder named `var`, containing all the files that the tryfile service can
   use, exists in the project root folder
6. If you intend to use the `old*quote` commands, a file named `csv_quotes.csv`
   lives in the project root folder

### Step 1

Run `make rustix` which builds the main rustix image

### Step 2

Run `make migration` which builds the db maintenence (for db migrations) image

### Step 3

Run `make setup` which generates a database password, launches rustix and the db
maintenence container (which then runs the db migrations) and then removes the
db maintenence container after running migrations.

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

If you see anything that could be improved (I'm sure there are many things),
please open an issue and/or PR! I'm open to feedback, and would love to
improve this project!
