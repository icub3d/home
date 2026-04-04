import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  Tool,
} from "@modelcontextprotocol/sdk/types.js";
import axios from "axios";

const HOME_URL = process.env.HOME_URL || "http://localhost:4001";
const HOME_API_KEY = process.env.HOME_API_KEY;

if (!HOME_API_KEY) {
  console.error("HOME_API_KEY environment variable is required");
  process.exit(1);
}

// Ensure the URL ends with /api for the backend routes
const baseURL = HOME_URL.endsWith("/api") ? HOME_URL : `${HOME_URL.replace(/\/$/, "")}/api`;

console.error(`Using base URL: ${baseURL}`);

const apiClient = axios.create({
  baseURL,
  headers: {
    "X-API-Key": HOME_API_KEY,
  },
});

const server = new Server(
  {
    name: "home-management-mcp",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

const TOOLS: Tool[] = [
  {
    name: "get_users",
    description: "Get all family members and their IDs.",
    inputSchema: {
      type: "object",
      properties: {},
    },
  },
  {
    name: "get_allowance_balances",
    description: "Get current allowance balances for all members.",
    inputSchema: {
      type: "object",
      properties: {},
    },
  },
  {
    name: "add_allowance_transaction",
    description: "Add a transaction to a user's allowance ledger. Amount is in cents (positive for credit, negative for debit).",
    inputSchema: {
      type: "object",
      properties: {
        user_id: { type: "string", description: "The UUID of the user" },
        amount: { type: "integer", description: "Amount in cents ($10.00 = 1000)" },
        description: { type: "string", description: "Reason for the transaction" },
      },
      required: ["user_id", "amount", "description"],
    },
  },
  {
    name: "list_chores",
    description: "List all chores and their current status.",
    inputSchema: {
      type: "object",
      properties: {},
    },
  },
  {
    name: "create_chore",
    description: "Create a new chore and optionally assign it to a user.",
    inputSchema: {
      type: "object",
      properties: {
        description: { type: "string", description: "What needs to be done" },
        assigned_to: { type: "string", description: "Optional UUID of the user to assign the chore to" },
        reward: { type: "integer", description: "Optional reward in cents" },
      },
      required: ["description"],
    },
  },
  {
    name: "toggle_chore",
    description: "Toggle a chore's completion status.",
    inputSchema: {
      type: "object",
      properties: {
        id: { type: "string", description: "The UUID of the chore" },
      },
      required: ["id"],
    },
  },
];

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: TOOLS,
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  console.error(`Calling tool: ${name} with args: ${JSON.stringify(args)}`);

  try {
    switch (name) {
      case "get_users": {
        const response = await apiClient.get("/users");
        return {
          content: [{ type: "text", text: JSON.stringify(response.data, null, 2) }],
        };
      }

      case "get_allowance_balances": {
        const response = await apiClient.get("/allowance/balances");
        return {
          content: [{ type: "text", text: JSON.stringify(response.data, null, 2) }],
        };
      }

      case "add_allowance_transaction": {
        const { user_id, amount, description } = args as {
          user_id: string;
          amount: number;
          description: string;
        };
        const response = await apiClient.post(`/allowance/${user_id}/transaction`, {
          amount,
          description,
        });
        return {
          content: [{ type: "text", text: `Transaction added successfully. New balance: $${(response.data.balance / 100).toFixed(2)}` }],
        };
      }

      case "list_chores": {
        const response = await apiClient.get("/chores");
        return {
          content: [{ type: "text", text: JSON.stringify(response.data, null, 2) }],
        };
      }

      case "create_chore": {
        const { description, assigned_to, reward } = args as {
          description: string;
          assigned_to?: string;
          reward?: number;
        };
        const response = await apiClient.post("/chores", {
          description,
          assigned_to,
          reward,
        });
        return {
          content: [{ type: "text", text: `Chore created successfully: ${response.data.id}` }],
        };
      }

      case "toggle_chore": {
        const { id } = args as { id: string };
        await apiClient.put(`/chores/${id}/toggle`);
        return {
          content: [{ type: "text", text: "Chore status toggled successfully." }],
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error: any) {
    console.error(`Error in tool ${name}:`, error.response?.data || error.message);
    const errorMessage = error.response?.data?.error || error.message;
    return {
      isError: true,
      content: [{ type: "text", text: `Error: ${errorMessage}` }],
    };
  }
});

async function run() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Home Management MCP server running on stdio");
}

run().catch((error) => {
  console.error("Fatal error in run():", error);
  process.exit(1);
});
