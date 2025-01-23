use std::collections::HashMap;
use tokio::{
    self,
    sync::mpsc::{channel, Receiver, Sender},
};
use uuid::Uuid;

const BATCH_SIZE: usize = 200;

struct Actor<H>
where
    H: Handler + Send + 'static,
{
    pid: String,
    handler: H,
    receiver: Receiver<ActorContext>,
}

impl<H> Actor<H>
where
    H: Handler + Send + 'static,
{
    fn new(handler: H) -> (Self, Sender<ActorContext>)
    where
        H: Handler + Send + 'static,
    {
        let (sender, receiver) = channel::<ActorContext>(BATCH_SIZE);
        (
            Self {
                pid: Uuid::new_v4().to_string(),
                handler,
                receiver,
            },
            sender,
        )
    }

    async fn handle_loop(mut self) {
        while let Some(ctx) = self.receiver.recv().await {
            self.handler.handle(&ctx);
        }
    }
}

struct ActorEngine {
    actors: HashMap<String, Sender<ActorContext>>,
}

impl ActorEngine {
    fn new() -> Self {
        Self {
            actors: HashMap::new(),
        }
    }

    async fn spawn(&mut self, handler: impl Handler + Send + 'static) -> String {
        let (actor, sender) = Actor::new(handler);
        let pid = actor.pid.clone();
        self.actors.insert(pid.clone(), sender);

        tokio::spawn(async move {
            actor.handle_loop().await;
        });

        pid
    }

    async fn send(&self, pid: impl Into<String>, message: impl Into<String>) {
        let sender = self.actors.get(&pid.into()).unwrap();
        sender
            .send(ActorContext::new(message.into()))
            .await
            .unwrap();
    }
}

trait Handler {
    fn handle(&mut self, ctx: &ActorContext);
}

struct ActorContext {
    message: String,
}

impl ActorContext {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

struct Helloer;

impl Handler for Helloer {
    fn handle(&mut self, ctx: &ActorContext) {
        println!("{}", ctx.message);
    }
}

#[tokio::main]
async fn main() {
    let mut engine = ActorEngine::new();
    let pid = engine.spawn(Helloer).await;
    engine.send(pid, "hello there!").await;
}
