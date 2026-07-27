return {
  "olimorris/codecompanion.nvim",
  opts = {
    adapters = {
      openrouter = function()
        return require("codecompanion.adapters").extend("openai_compatible", {
          env = {
            url = "https://openrouter.ai/api/v1",
            api_key = "OPENROUTER_API_KEY",
          },
          schema = {
            model = {
              default = "poolside/laguna-s-2.1:free",
            },
          },
        })
      end,
    },

    strategies = {
      chat = {
        adapter = "openrouter",
      },
      inline = {
        adapter = "openrouter",
      },
      cmd = {
        adapter = "openrouter",
      },
    },
  },
}
