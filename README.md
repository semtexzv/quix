# Quix
Distribution layer for Actix.
Aims to follow conventions set up by actix, and implement distribution capabilities, similar to erlang's [dist](https://erlang.org/doc/apps/erts/erl_dist_protocol.html).

## Usage 
See [docs](https://docs.rs/quix)


Protobuf:
```protobuf
message M1 {

}
service Exec{
  rpc Method(M1) returns(M1);
}
```

Code:
```rust
#[derive(quix::DynHandler)]
#[dispatch(Method)]
pub struct Act {}

impl Actor for Act {

    type Context = Process<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let pid : Pid<Self> = ctx.pid();
        pid.do_send(M1 { v: 0 });
    }
}
impl Handler<Method> for Act { 
    type Response = Result<M1, DispatchError>;
... 
}
```

### Processes
In order to better handle distribution, we will introduce a concept of a process, which is just an identified actor.
This actor is not referneced through the `Addr<A>` struct, but rather through `Pid<A>`, which can be: 
1. `Pid::Local` Wraps `Addr<A>` and transparently passes messages to this addr
2. `Pid::Remote` contains global process ID, and uses local process registry to determine where to send the message

The `Pid<A>` can be obtained in 2 ways: 
1. Using `Process<A>` instead of `Context<A>` - New context type for actors, which have stable identity
2. Receiving a message containing a `PidProto` - Pids are transparent, and can be sent between nodes.
The distribution subsystem should handle node lookup internally, thorugh the node-local registry.

### Messages
We use protobuf for defining the message types, and for generating necessary serialization and deserialization code.

The `Service` in protobuf file defines a set of m`rpc` methods. We take these `rpc` entries, and generate message defintions
from them. Eg:

```protobuf
message M1 {

}
service Exec{
  rpc Method(M1) returns(M1);
}
```
will generate something like this:
```rust

pub struct Method(pub M1);

impl actix::Message for Method {
    type Result = Result<M1, DispatchError>;
}

impl quix::derive::RpcMethod for Method {
    const NAME: &'static str = "quix.net.Exec.method";
    const ID: u32 = 529753007;

    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> { ... }
    fn read(b: impl bytes::Buf) -> Result<Self, DispatchError> { ... }

    fn read_result(b: impl bytes::Buf) -> Self::Result { ... }
    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> { ... }
}
```

If you want to process the `Method` messages, you just have to implement `Handler<Method>` and annotate
your `Process` actor with: 

```rust
#[derive(DynHandler)]
#[dispatch(Method)]
pub struct YourActor {}
```
Then, you can either register your as a Node-global handler by sending a message to the `NodeController` actor,
or you can just send your `Pid<Self>` serialized into `PidProto`.

### ProcessRegistry
Is a node-local singleton, which contains a map of actors. Process is automatically registered and unregistered from this 
registry when started and stopping. The registry uses gossip protocol to get rough image of whole cluster.

### Message dispatching
Each message which is passable across network boundaries must be serializable using protobuf.

The `DynHandler` trait is responsible for creating a `Dispatcher` which performs message serialization and
dispatching. It can be implemented by a proc macro:

```rust
#[derive(quix::DynHandler)]
#[dispatch(M1, M2)]
pub struct Act {}

impl Actor for Act {
    type Context = Process<Self>;
}
impl Handler<M1> for Act { ... }
impl Handler<M2> for Act { ... }
impl Handler<M3> for Act { ... }
```

The `Act` actor can locally respond to `M1`,`M2` and `M3` but only respond to remotely submitted `M1` or `M2`