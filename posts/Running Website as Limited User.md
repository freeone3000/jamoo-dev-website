Previously, the website (with no user input, no interaction, that serves static files) was running *as my user* 😱, or worse, under Docker as system 😱🙀🫨 In order to fix this, some changes needed to be made.

[This RedHat documentation](https://www.redhat.com/sysadmin/container-systemd-persist-reboot) will be used as a guide and touchstone throughout. My dev and deploy system, however, is running Debian Bookworm, so my adaptations will follow.
## Swapping Podman for Docker
Docker containers run using an agent as root. In order to drastically limit
First, the [Dockerfile](https://github.com/freeone3000/jamoo-dev-website/blob/master/Dockerfile) needed to be changed to use "docker.io/" as is package prefix. Simple change, just change the first line in the builder
<pre><code class="language-Dockerfile">FROM <mark>docker.io/</mark>rust:1.74-alpine3.18 as builder
RUN apk add --no-cache musl-dev
...
</code></pre>

and make the same change in the run file,
<pre><code class="language-Dockerfile">FROM <mark>docker.io/</mark>alpine:3.17
WORKDIR /website
...
</code></pre>

I currently do about one build every three to four days, so having my build server act as my local cache is sufficient to keep myself under Docker's very generous free limitations.

Next, `sed -i.bak s@docker@podman@ run.sh`.  I don't use Swarm, Kubernetes, or any other advanced deployment configuration, so that simple swap will get me the new podman runtime configuration to use.
## Installing the systemd unit
### Creating the Unit File
```shell
podman run --rm --name jamoo.dev -p 8080:3000 $PWD/posts:/website/posts $PWD/static/:/website/static -v $PWD/templates:/website/templates jamoo.dev &
podman generate systemctl --new -n --files
podman kill jamoo.dev
```
This will output `container-jamoo.dev.service`, the exact unit file we want. For your and my sanity, we will set this as the shell var `svc=container-jamoo.dev.service` 
We copy `$svc` into `/home/webuser/.config/systemd/user/`, and then run `systemctl --user daemon-reload` to refresh the systemd daemon (and check dbus is working). Verify the unit is valid using `systemd-analyze --user verify $svc`
### Configuring the User
Systemd requires dbus, dbus requires the user to be a *real user*, login and shell and everything. So, we're going to make a regular user, and give us a login through ssh:
```zsh
sudo adduser webuser
# prevent cli-interactive logins
sudo passwd -l webuser 

fn=$(mktmp -p /tmp/webuser-install)
ssh-keygen -f "$fn" -P '' -q
sudo mkdir -p /home/webuser/.ssh
echo $(cat "$fn.pub") | sudo tee -a /home/webuser/.ssh/authorized-keys
# fix permissions on .ssh dir
sudo chown -R webuser:webuser /home/webuser/.ssh && sudo chmod -R u=rwX,o-rwx,g-rwx /home/webuser/.ssh

# enable persistent logins
sudo loginctl enable-linger webuser
```
The above is a fairly standard public key authentication setup. `passwd -d` would also work, except that it allows the user to later reset their own password, which we do *not* want as it allows a method login persistence. (Remember that  `webuser` can't read the private key we generated earlier!)

One addition is running `loginctl enable-linger webuser`, which will enable the user to persist units without having a persistent login shell. This is important for a webserver!
### Running the Service
Unlike docker, podman containers are created *per user*. This means that we need to re-build the image as our target user with `podman build -t jamoo.dev $CODE_DIR`.

After doing so, `systemctl --user start $svc` will bring up our service! So now we reach for `systemctl --user enable --now $svc` and...
<pre><code class="language-shell"><span style="color: red">Failed to enable unit: Unit file /home/webuser/.config/systemd/user/default.target.wants/container-jamoo.dev.service does not exist.
</span></code></pre>
Hmm. This *should* exist, but doesn't. Checking permissions, `webuser` doesn't actually have write permissions to its own .cache directory, and this is the error that results from that! (It was created by another user; non-linearity strikes again.)

Running `sudo setfacl -R -m u:webuser:rwX /home/webuser/.config` as my regular user fixes the permissions problem. (Obviously, "webuser" does not and [should not]([CVE - Search Results (mitre.org)](https://cve.mitre.org/cgi-bin/cvekey.cgi?keyword=sudo)) have sudo rights!) Running `systemctl --user enable --now $svc` ensures that the service meets the three criteria:
1. Running as a limited user, no privilege escalation possible
2. On system startup
3. That restarts on failure
4. With very limited remote access
While still allowing me to access the user when needed via ssh.
## Automating
Since I've still got the heart of a devops gal (aka, a boatload of tenacity), we're going to automate this! This blog post was actually written from the automation script, debugging it line-by-line until they worked. One change that wasn't covered was automatic ssh key generation for the target, which is completely unnecessary but kinda cool (and allows the script to be run by root, or arbitrary other users with sudo without the need for keysharing). [install.sh lives here](https://github.com/freeone3000/jamoo-dev-website/blob/master/install.sh).