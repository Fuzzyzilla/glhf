# GL, HF
A quirky lil OpenGL ES 3.X wrapper crate making excessive use of typestate and the borrow checker to ensure correct API
usage at compile time with *zero* runtime cost.

**Why ES 3?** Unfortunately, this is the lingua-franca of legacy graphics. Apps targeting GL ES 3 can compile to
the Web, to Android, and even to iOS. While efforts exist to bridge this gap, for my purposes this is the tool I need.

Along with regular rust-ification of the API (splitting up the monolithic `GLenum` with many disjoint enums and
eliminating invalid argument combinations at the type level), two primary strategies are at play:

## Typestate
Some types in OpenGL require passing certain runtime checks before becoming usable, or have important runtime state
determined by the ways in which they are accessed - for example, when a texture name is first bound to a texture target,
it goes from an uninitialized state to being permanently associated with that texture target type; a framebuffer must
pass "Completeness" checking; a program must be linked.

This is codified in a "Typestate" API, where the types of variables change depending on the statically-known dataflow
they are used in.

Along with this, the bindpoints themselves (`glBindBuffer` and the likes) are written in a typestate manner. For example,
It must be statically known that a *non-null* buffer is bound to the `GL_ELEMENT_ARRAY_BUFFER` buffer in order to call
`gl.draw.elements(...)`.

## Borrow Checker
The relationship between objects in OpenGL is sometimes complex and oftentimes difficult to find documentation for. This
crate expresses those relationships by projecting them as rust borrows.

To execute a command, you must prove (at compile time) that you hold all the necessary resources. If any call is made
that may sneakily change those resources out from under you, the references are invalidated, creating an error at compile
time instead of a hard-to-track bug at runtime.

```rust
// Bind our texture to `TEXTURE_2D`...
let active_texture = gl.texture.d2.bind(&awesome_texture);

// (Whoops! glActiveTexture switches out every texture binding!)
gl.texture.unit(1);

// ...try to update our previously-bound `awesome_texture`
active_texture.mag_filter(glhf::texture::Filter::Linear);
```

```rust
error[E0499]: cannot borrow `gl.texture` as mutable more than once at a time
|
| let active_texture = gl.texture.d2.bind(&self.shadow_texture);
|                      ------------- first mutable borrow occurs here
| gl.texture.unit(1);
| ^^^^^^^^^^ second mutable borrow occurs here
|
| active_texture.mag_filter(glhf::texture::Filter::Linear);
| -------------- first borrow later used here
```

## Cool tricks
I just think they're neat :3

```rust
let [a, handful, of, textures] = gl.new.textures();
```

## Non-goals
* Automatic resource management - object handles do not have drop glue, and must be manually deleted.
* Preventative error checking - other than the not-unsubstantial compile-time error prevention, runtime state is not
  queried to ensure GL calls will not error. 
* Full safety - unsafe APIs are required to make use of this library, as even something as simple as `gl.draw.arrays`
  may invoke UB. To to greatest extent possible howerver, UB is documented and some is prevented at compile time.
* Object-oriented API - All handles are thin-wrappers around `NonZero<GLuint>`, and all state APIs are ZSTs.
