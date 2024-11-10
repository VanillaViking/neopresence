
# Neopresence

A plugin for discord rich presence in neovim

![image](https://github.com/user-attachments/assets/02cbfb99-7480-4202-b19c-25fdd493aa80)



features:
* language detection
* session diff of additions & deletions
* remote repository detection

# Installation

This plugin can be easily installed using cargo:
```bash
cargo install neopresence
```

Make sure to add `.cargo/bin` to your PATH variable

Next, add this to your Neovim config:
```lua
local id, err = vim.lsp.start_client({
  name = 'neopresence',
  cmd = {'neopresence'},
 })

vim.api.nvim_create_autocmd({"FileType"}, {
  pattern = {"*"},
  callback = function()
    vim.lsp.buf_attach_client(0, id) 
  end
})
```
