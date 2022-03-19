# r6v3

A discord bot, which currently features the following commands:
- `~ping`: Just replies `Pong!`. Required permission: `/ping`
- `~start <instance>`: Starts an azure instance as configured in `config.toml`. Required permission: `/{instance}/start`
- `~stop <instance>`: Stops an azure instance as configured in `config.toml`. Required permission: `/{instance}/stop`

Configuration files:

- `config.toml`: Configuration of the discord bot and azure instances
```toml
discord_token = "<TOKEN>"

# For configuring the azure application
[azure]
directory = "<TENANT ID>"
client = "<CLIENT ID>"
cert_path = "azure.crt"
cert_key = "azure.key"

# Lets configure a minecraft server with name "mc"
[servers.mc]
# Gets executed on the remote vm on start
start_script = "scripts/mc/start"
# Gers executed on the remote vm on stop
stop_script = "scripts/mc/stop"

# Configuration of the azure instance
[servers.mc.vm]
name = "mc001"
rg = "mcRG"
sub = "<SUBSCRIPTION ID>"

# Additionally, we can configure a teamspeak server with name "ts"
[servers.ts]
start_script = "scripts/ts/start"
stop_script = "scripts/ts/stop"

[servers.ts.vm]
name = "ts001"
rg = "tsRG"
sub = "<SUBSCRIPTION ID>"
```

- `permissions.toml`: Definition of roles
```toml
owner = ["*"]
mc = ["/mc/start", "/mc/stop"]
ts = ["/ts/start", "/ts/stop"]
start = ["/*/start"]
```

- `users.toml`: Assign roles to discord users
```toml
# User ID of Bot owner
123456789 = ["owner"]

# User ID of Friend
987654321 = ["mc", "ts"]
```

- `groups.toml`: Assign roles to discord roles
```toml
# Role ID of the admin role
121212121 = ["owner"]
```
