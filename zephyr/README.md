# Building Zephyr examples for zeroHETI

- General dependencies from the [official docs.](https://docs.zephyrproject.org/latest/develop/getting_started/index.html)

- Create a dedicated directory for a workspace, i.e., `workspace_dir`. Create and activate a new virtual environment in the directory with 
```
python3 -m venv workspace_dir/.venv
source workspace_dir/.venv/bin/activate
```


- Initialize the workspace as Zephyr workspace with 
```sh
cd workspace_dir
west init
```

- Specify zeroHETI as tyhe manifest repository to allow Zephyr to find the local files. Do this with 
```
west config manifest.path \<path/to/zeroheti\>
```

- Test setup by building an example for the `hetiboard` target, e.g.,
```
cd zephyr/samples/hello_world
west build -b 
```
