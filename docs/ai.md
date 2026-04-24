# AI driven creativity

Other than the vague idea (which is not overly original TBH), almost all of the creative parts of this project have been done by AI:

- Name: Gemini 2.5 (I described the project then asked for a subtly Touhou inspired name. I picked the one that was least bad.)
- Story attributes: Gemini 2.5 (I gave it my half baked ideas, the selected attributes were a mix of its suggestion and one of its [references](https://storyenginedeck.com/blogs/news/guide-to-card-creation).)
- Card content: Gemini 2.5 (I gave it the attributes and asked for "a large deck of cards", it gave me 100 cards in total, which was suspiciously the same number of Mifare Classic cards I ordered.)
- USB VID/PID: Copilot/GPT-5 mini (I asked it to generate some fun suggestions since this is just a single use thing. It did and I picked one.)
- Board side panel art: Nano Banana Pro (I gave it the size of the panel and a generic "magic/arcane meets tech/cyberpunk" prompt, being more specific if I wanted certain things to appear. Generated images needed a little bit of cleaning to be suitable for laser engraving.)
- Board top panel art: Nano Banana Pro (I told it I wanted an "arcane meets cybperpunk" design that fits inside an octagon. It took a few iterations of modifying the generated image to end up with something suitable.)
- Card art template: Nano Banana Pro (I asked it to generate a card template with a specific layout and space for art. [What it gave](../cards/template.png) was pretty much ideal.)
- Cart art: Nano Banana Pro (Via [`controller gen-card-art`](../controller/src/card_art_generation.rs) I gave it the card template and the card content, and asked it to replace the placeholder text and generate art for each card. Some tweaks were needed for particular cards and some took a few iterations to get something I liked.)
- GitHub Copilot was used for quite a bit of the software, mostly text completion and some narrowly scoped tasks. There was very little that could be used without some modification.
- GitHub Copilot generated the code for most of the visual effects using the WS2812B ring, at least the more complex ones. It was prompted with the [`BoardLightingEffect`](../controller/src/logic/actors/board/ambient_light/effects/mod.rs) trait and a general idea of what I wanted it to look like.
- Printer box art: Nano Banana Pro
