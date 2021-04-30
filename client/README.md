Install `wasm-pack`, as described here: [https://yew.rs/docs/getting-started/project-setup/using-wasm-pack](https://yew.rs/docs/getting-started/project-setup/using-wasm-pack).

Build from within the  `client` directory:

```shell script
wasm-pack build --target web --out-name wasm --out-dir ../server/static
```

or

``` shell
trunk build -d ../server/static/ 
```
Wasm package will be published in `server/static` directory.

