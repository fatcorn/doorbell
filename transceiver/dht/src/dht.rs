use doorbell_common::node::NodeInfo;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Node (pub(crate) NodeInfo);

impl From<&[u8]> for Node {
    fn from(source: &[u8]) -> Self {
        Self::try_from_slice(source).unwrap()
    }
}

impl Node {
    pub fn to_vec(self) -> Vec<u8> {
        borsh::to_vec(&self).unwrap()
    }
}

mod tests {
    use doorbell_common::config::Config;
    use doorbell_common::node::NodeInfo;
    use crate::dht::Node;

    #[test]
    fn test_node_fn() {
        let config = Config::default();
        let node = Node {
            0: NodeInfo::get_from_config(&config)
        };

       let ser_ret =  node.to_vec();
        println!("{:?}", ser_ret);

        let deser_ret = Node::from(ser_ret.as_slice());
        println!("{:?}", deser_ret);
    }
}