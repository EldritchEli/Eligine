use VulcanEngine_0::bevy_app::app::{self, AssetList};
const ASSET_PREFIX : &str = "";
fn main() {
    let list = AssetList::new(vec![
        format!("{ASSET_PREFIX}living_room/Chair.glb"),
        format!("{ASSET_PREFIX}living_room/Drawer.glb"),
        format!("{ASSET_PREFIX}living_room/Couch.glb"),
        format!("{ASSET_PREFIX}living_room/Laptop.glb"),
        format!("{ASSET_PREFIX}living_room/Curtain.glb"), 

    ]);
    app::create_app(list).run();
    
}