use super::{subscriber_email::SubscriberEmail, subscriber_username::SubscriberUsername};

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub username: SubscriberUsername,
}
