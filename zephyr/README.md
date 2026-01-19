# Building Zephyr examples for zeroHETI

- General dependencies from the [official docs.](https://docs.zephyrproject.org/latest/develop/getting_started/index.html)

1. Create a dedicated directory for a workspace, i.e., `workspace_dir`. Create and activate a new virtual environment in the directory with 
```
python3 -m venv workspace_dir/.venv
source workspace_dir/.venv/bin/activate
```

2. Install `west` within new virtual environment: `pip install west`

3. Initialize the workspace as Zephyr workspace with 
```sh
cd workspace_dir
west init
west update
```

4. Export a Zephyr CMake package. This allows CMake to automatically load boilerplate code required for Zephyr applications.
```
west zephyr-export
```

5. Install Python dependencies:
```
west packages pip --install
```

After this general setup, running `west boards` from `workspace_dir` should print out the supported boards. To add our board:

1. Specify zeroHETI as the manifest repository to allow Zephyr to find the local files:
```
west config manifest.path <relative/path/to/zeroheti>
west update
```
`west boards` should now print `hetiboard` as the last entry.

2. Test setup by building an example for the `hetiboard` target, e.g.,
```
cd zephyr/samples/hello_world
west build -b hetiboard
```
