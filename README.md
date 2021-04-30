## Full-stack Rust with WebAssembly

# Look Ma, No JavaScript !!! 

My very first Rust project (implementation of the "Connect 5" game), I used as a learning tool.

![img.png](img.png)

You can find (and eventually - learn, if you are not there yet) how to:
### Client:
- build Web client in Rust, without a single line of JavaScript, using  [Yew](https://github.com/yewstack/yew) WebAssembly framework
- use [yew-router](https://github.com/yewstack/yew/tree/master/packages/yew-router) for page navigation
- multithreading components and message passing with Yew Agents
- StorageService to keep session data
- utilise a CSS framework ([Bulma](https://bulma.io))

### Server:
- Use [Actix Web](https://github.com/actix/actix-web) web framework to implement REST API
- [Diesel](https://diesel.rs) ORM with SQLite as default DB 
- Session cookies
- Integration testing for the REST API
- Mocking functions (db calls) in unit tests 
