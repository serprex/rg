rg: Real Rust Roguelike, goal: gg

So I was winding down from working on [openEtG](https://github.com/serprex/openEtG) & in my EtG fanaticism it was suggested I make an EtG roguelike

Cue arguments about roguelike vs diablolike. Art was developed, I started to hate myself for using nodejs, & I did what I always do

I ran away. I devolved into my protostate. I wanted something _real_ I wanted a _real_ roguelike

I don't think all roguelikes need be terminal but damnit that's what I'm going to do. & in Rust because Rust rocks

& ncurses doesn't so I'll write my own [terminal library](https://github.com/serprex/x1b)

But I tried to shoehorn a game's type system to Rust's. Big mistake. Games are dynamic. I was slowly evolving the game into basically being a bunch of hashmaps

Oh my that'd be bad. But behold, Entity Component Systems. I looked at them from afar, wondering 'Are they suckless? Or are they a wannabe-silverbullet?'

NEITHER. But they suck less than hashmaps so I picked up [specs](https://github.com/slide-rs/specs) because it was suppose to be fast & [kvark](https://github.com/kvark) seems cool even if I feel like I annoy him

ANYWAYS, by then I'd set the codebase down for a few months after breaking it while trying to figure out how to have 2 different types of enemies share code. I rewrote it in specs (commit 9e5v8fc2f444feb2701d6ac7e6779fde42d0d9d) in like a day & oh my we lost all our cool features like listing out the types of all the things

That's a good sign. Now I'm adding back in stuff like enemy types that can't attack & then attacking & shooting & now the enemies strike back & now there's an inventory system where after you pick stuff up you can't really access it in any way but it's there the data is out there you just have to close your eyes & imagine the registers flipping their bits in unison here comes the chorus it's loud & my eyes are shining behind these lids can't you see with your eyes gone closed

I digress. Somewhere in there I realize I still want people to play this, I think maybe ssh is the answer, just like [dpc](https://github.com/dpc) did with [rhex](https://github.com/dpc/rhex), but ssh is confusing & I suck so I decided instead to implement [x1b.js](https://github.com/serprex/x1b.js) which brings the concept of teletype full circle by piping rg's binary to websockets which a javascript program uses to render the terminal state in the browser

Eventually I'd like to move to non blocking anonymous named pipes so that multiple people can all take part in the same game, getting their own unique view. But that's something which I don't feel like smashing my head against when there's so much more to do like an inventory screen

This was suppose to be a readme that serves as a tutorial to specs because they'd like some tutorials but I suck at writing instructively