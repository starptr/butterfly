terraform {
	required_version = ">= 1.3.0"
	required_providers {
		cloudflare = {
			source = "cloudflare/cloudflare"
			version = "4.0.0"
		}
	}
}

provider "cloudflare" {
	api_token = var.cloudflare_api_token
}

variable "cloudflare_api_token" {
  type      = string
  sensitive = true
}

variable "cloudflare_account_id" {
	type      = string
	sensitive = true
}

resource "cloudflare_worker_script" "butterfly" {
	account_id = var.cloudflare_account_id
	name    = "butterfly-terra"
	content = file("build/worker/shim.mjs")
	webassembly_binding {
		name  = "butterflywasm"
		module = filebase64("build/worker/index.wasm")
	}
}
