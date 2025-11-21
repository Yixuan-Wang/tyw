# `tyw py`

Utilities for Python development.

## Configuration

```toml
[py]
env.home = "<path>" # path to your Python global venv home
```

## Environment

[`conda`](https://docs.conda.io/en/latest/) and friends are falling out of favor.
However [`pdm`](https://pdm.fming.dev/) and [`uv`](https://astral.sh/uv/) fails to tackle the paradigm in academia where the virtualenvs must be stored in another directory to the code,
or a global virtualenv is preferred.

### `use` and `sel`

`use` and `sel` are two commands that can be used to activate different Python environments.

```bash
# print the activation command for a virtualenv
$(tyw py use <name>) # the Python virtualenv named <name>
$(tyw py use       ) # find the venv or .venv in the nearest parent
$(tyw py sel       ) # fuzzyfind available Python virtualenvs with `fzf`
```

To activate the environment, eval the output of this command.

```bash
eval "$(tyw py use)"
```

### `list`

List all available Python virtualenvs.

```bash
tyw py list
```
