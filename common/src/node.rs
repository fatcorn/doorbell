
type Pubkey = [u8; 32];

pub struct UserInfo {

    pub pubkey: Pubkey,

    pub nick_name: String,

    pub address: String,

    pub sig_path: String,

}