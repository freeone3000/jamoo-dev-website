
#!/usr/bin/env zsh
set -e

docker run --rm --name jamoo.dev -p 8080:3000 -v $PWD/posts:/website/posts -v $PWD/static:/website/static -v $PWD/templates:/website/templates jamoo.dev
