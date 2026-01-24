use text_components::TextComponent;

fn main() {
    use std::io::{Write, stdin, stdout};
    let mut s = String::new();
    print!("/tellraw @s ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    let component = TextComponent::from_snbt(&s);
    match component {
        Ok(component) => {
            println!("{:?}", component);
            println!("{:p}", component)
        }
        Err(e) => eprintln!("{}", e),
    }
    // ["\"Howdy!\"", { text:"\nThis is a text component!\n", color:'blue', "bold":1b, italic:true }, {text:"Texto" , type: "translatable", fallback:"lol\n", translate:"lmao"}, {sprite:"items/iron_sword"}, "\n", {object:"player", player:{name:"MrMelther"}, hover_event:{action:"show_text",value:{text:"Send msg to MrMelther"}}, click_event:{action:"suggest_command", command:"/msg MrMelther "} }, {object:"player", player:{properties:[{name:"textures", value:"[Put your base64 texture here!]"}]}}]
    // {text:"Test",extra:[" lmao"]}
}
