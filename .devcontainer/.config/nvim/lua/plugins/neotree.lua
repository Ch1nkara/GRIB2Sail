# Fix the delete: deleting instead of trashing
return {
  {
    "nvim-neo-tree/neo-tree.nvim",
    opts = {
      filesystem = {
        use_libuv_file_watcher = true,
      },
      commands = {
        delete = function(state)
          local node = state.tree:get_node()
          vim.fn.delete(node.path, "rf")
        end,
      },
    },
  },
}
