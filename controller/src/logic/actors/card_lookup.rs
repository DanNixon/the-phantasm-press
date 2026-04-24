use crate::{
    card::CardLibrary,
    logic::{CardsInReaders, ReaderState},
};
use kameo::{
    Actor,
    actor::{ActorRef, WeakActorRef},
    prelude::{Context, Message},
};
use kameo_actors::message_bus::{MessageBus, Publish};
use log::info;
use protocol::{CardId, CardReader, CardReaderState};
use std::{collections::HashMap, convert::Infallible};

pub(crate) struct CardLookupActor {
    library: CardLibrary,
    cards: HashMap<CardReader, CardId>,
    event_bus: WeakActorRef<MessageBus>,
}

pub(crate) struct CardLookupActorArgs {
    event_bus: WeakActorRef<MessageBus>,
}

impl CardLookupActorArgs {
    pub(crate) fn new(event_bus: WeakActorRef<MessageBus>) -> Self {
        Self { event_bus }
    }
}

impl CardLookupActor {
    fn cards_in_readers(&self) -> CardsInReaders {
        CardsInReaders(
            [
                CardReader::One,
                CardReader::Two,
                CardReader::Three,
                CardReader::Four,
            ]
            .iter()
            .map(|reader| {
                let state = match self.cards.get(reader) {
                    Some(card_id) => self
                        .library
                        .get(card_id)
                        .map(|c| ReaderState::Card(c.clone()))
                        .unwrap_or(ReaderState::UnknownCard),
                    None => ReaderState::NoCard,
                };
                (*reader, state)
            })
            .collect(),
        )
    }
}

impl Actor for CardLookupActor {
    type Args = CardLookupActorArgs;
    type Error = Infallible;

    async fn on_start(args: Self::Args, _: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(Self {
            library: CardLibrary::load().unwrap(),
            cards: HashMap::new(),
            event_bus: args.event_bus,
        })
    }
}

impl Message<CardReaderState> for CardLookupActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: CardReaderState,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match msg.state {
            Some(card_id) => {
                info!("Card {card_id} present in reader {:?}", msg.reader);
                self.cards.insert(msg.reader, card_id);
            }
            None => {
                info!("Card removed from reader {:?}", msg.reader);
                self.cards.remove(&msg.reader);
            }
        }

        let cards = self.cards_in_readers();
        info!("Cards in readers: {:?}", cards);

        self.event_bus
            .upgrade()
            .unwrap()
            .tell(Publish(cards))
            .await
            .unwrap();
    }
}
