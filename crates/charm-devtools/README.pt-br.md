# charm-devtools

Middleware de desenvolvimento para models Bubble Tea. Ele registra um histórico limitado de mensagens, captura o estado renderizado e mostra um painel de inspeção ao pressionar **F12**. O painel também permite rolagem com **Up/Down** e troca de seção com **Tab**.

```rust
use charmed_bubbletea::{Cmd, Message, Model};
use charm_devtools::{middleware, ComponentNode, DevInspect};

struct App;
impl Model for App {
    fn init(&self) -> Option<Cmd> { None }
    fn update(&mut self, _: Message) -> Option<Cmd> { None }
    fn view(&self) -> String { "app".to_owned() }
}
impl DevInspect for App {
    fn inspect(&self) -> ComponentNode { ComponentNode::root("App") }
}

let wrapped = middleware(App);
assert_eq!(wrapped.view(), "app");
```

O histórico conserva as 100 mensagens mais recentes. Todas as mensagens, inclusive as teclas de navegação do inspetor, são encaminhadas ao model interno. `DevState::current_state()` guarda a última view do model.

## Breaking change em relação ao Go

O Go usa reflexão para descobrir models aninhados automaticamente. Rust não oferece reflexão equivalente em runtime; portanto, models devem implementar `DevInspect` para descrever a árvore de componentes. A implementação padrão informa somente o tipo raiz. Essa é uma alteração incompatível deliberada. Além disso, `Message` do Bubble Tea apaga o tipo concreto e não expõe o valor do payload; os registros usam o tipo apagado e sua representação pública `Debug`.
