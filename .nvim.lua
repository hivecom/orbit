local conform = require("conform")

if vim.fn.executable("vp") == 1 then
  conform.formatters.vp_fmt = {
    command = "vp",
    args = { "fmt", "$FILENAME" },
    stdin = false,
  }

  local target_fts = { "javascript", "typescript", "vue" }
  for _, ft in ipairs(target_fts) do
    conform.formatters_by_ft[ft] = { "vp_fmt" }
  end
else
  vim.notify(
    "[.nvim.lua] 'vp' not found in PATH. Skipping project formatters.",
    vim.log.levels.WARN
  )
end
