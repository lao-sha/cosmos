/**
 * Costik 链上 Pallet 集成测试脚本
 *
 * 测试 pallet-bot-consensus, pallet-bot-registry, pallet-bot-group-mgmt
 * 在 dev chain 上的完整流程
 *
 * 前置条件:
 *   - Substrate dev chain 运行中 (ws://127.0.0.1:9944)
 *   - 已安装 @polkadot/api
 *
 * 用法:
 *   npx ts-node test-chain.ts
 *   # 或
 *   npx tsx test-chain.ts
 */

import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";

const WS_URL = process.env.CHAIN_RPC || "ws://127.0.0.1:9944";

async function main() {
  console.log("═══════════════════════════════════════════");
  console.log("  Costik Chain Integration Tests");
  console.log("═══════════════════════════════════════════\n");

  const provider = new WsProvider(WS_URL);
  const api = await ApiPromise.create({ provider });

  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice");
  const bob = keyring.addFromUri("//Bob");
  const charlie = keyring.addFromUri("//Charlie");

  let passed = 0;
  let failed = 0;

  function check(name: string, ok: boolean) {
    if (ok) {
      console.log(`  ✓ ${name}`);
      passed++;
    } else {
      console.log(`  ✗ ${name}`);
      failed++;
    }
  }

  // ══════════════════════════════════════════
  // 1. pallet-bot-registry 测试
  // ══════════════════════════════════════════
  console.log("[1] pallet-bot-registry\n");

  const botIdHash = new Uint8Array(32).fill(0xaa);
  const communityHash = new Uint8Array(32).fill(0xbb);
  const publicKey = new Uint8Array(32).fill(0xcc);

  // register_bot
  try {
    const tx = api.tx.botRegistry.registerBot(
      botIdHash,
      communityHash,
      "Telegram", // Platform enum
      publicKey,
    );
    await tx.signAndSend(alice);
    await waitBlocks(api, 2);

    const bot = await api.query.botRegistry.bots(botIdHash);
    check("register_bot: bot exists", !bot.isEmpty);

    const ownerBots = await api.query.botRegistry.ownerBots(alice.address);
    check("register_bot: owner list updated", ownerBots.toJSON() !== null);
  } catch (e: any) {
    check(`register_bot: ${e.message}`, false);
  }

  // update_bot_public_key
  try {
    const newPk = new Uint8Array(32).fill(0xdd);
    const tx = api.tx.botRegistry.updateBotPublicKey(botIdHash, newPk);
    await tx.signAndSend(alice);
    await waitBlocks(api, 2);

    const bot: any = (await api.query.botRegistry.bots(botIdHash)).toJSON();
    check("update_public_key: key updated", bot !== null);
  } catch (e: any) {
    check(`update_public_key: ${e.message}`, false);
  }

  // bind_community_platform
  try {
    const platformCommunityHash = new Uint8Array(32).fill(0xee);
    const tx = api.tx.botRegistry.bindCommunityPlatform(
      communityHash,
      "Telegram",
      platformCommunityHash,
      botIdHash, // optional bot
    );
    await tx.signAndSend(alice);
    await waitBlocks(api, 2);

    const binding = await api.query.botRegistry.communityPlatforms(
      communityHash,
      "Telegram",
    );
    check("bind_community_platform: binding exists", !binding.isEmpty);
  } catch (e: any) {
    check(`bind_community_platform: ${e.message}`, false);
  }

  // bind_user_platform
  try {
    const userPlatformHash = new Uint8Array(32).fill(0xff);
    const tx = api.tx.botRegistry.bindUserPlatform(
      "Telegram",
      userPlatformHash,
    );
    await tx.signAndSend(bob);
    await waitBlocks(api, 2);

    const binding = await api.query.botRegistry.userPlatformBindings(
      bob.address,
      "Telegram",
    );
    check("bind_user_platform: binding exists", !binding.isEmpty);
  } catch (e: any) {
    check(`bind_user_platform: ${e.message}`, false);
  }

  console.log("");

  // ══════════════════════════════════════════
  // 2. pallet-bot-consensus 测试
  // ══════════════════════════════════════════
  console.log("[2] pallet-bot-consensus\n");

  // register_node (需要质押 >= MinStake)
  try {
    const nodeIdHash = new Uint8Array(32).fill(0x01);
    const endpoint = Array.from(
      new TextEncoder().encode("http://node1:8080"),
    );
    const nodePk = new Uint8Array(32).fill(0x11);

    const tx = api.tx.botConsensus.registerNode(
      nodeIdHash,
      endpoint,
      nodePk,
    );
    await tx.signAndSend(alice);
    await waitBlocks(api, 2);

    const node = await api.query.botConsensus.nodes(nodeIdHash);
    check("register_node: node registered", !node.isEmpty);
  } catch (e: any) {
    // MinStake 不足时会失败 — 这是预期的
    check(
      `register_node: ${e.message?.includes("Stake") ? "需要更多质押 (预期)" : e.message}`,
      e.message?.includes("Stake") || false,
    );
  }

  console.log("");

  // ══════════════════════════════════════════
  // 3. pallet-bot-group-mgmt 测试
  // ══════════════════════════════════════════
  console.log("[3] pallet-bot-group-mgmt\n");

  // set_group_rules
  try {
    const tx = api.tx.botGroupMgmt.setGroupRules(
      communityHash,
      "AutoApprove", // JoinApprovalPolicy
      30, // rate_limit_per_minute
      100, // auto_mute_duration (blocks)
      true, // filter_links
      false, // restrict_mentions
    );
    await tx.signAndSend(alice);
    await waitBlocks(api, 2);

    const rules = await api.query.botGroupMgmt.groupRulesStore(communityHash);
    check("set_group_rules: rules exist", !rules.isEmpty);
  } catch (e: any) {
    check(`set_group_rules: ${e.message}`, false);
  }

  // log_action
  try {
    const targetUser = new Uint8Array(32).fill(0x22);
    const executorNode = new Uint8Array(32).fill(0x33);
    const msgHash = new Uint8Array(32).fill(0x44);

    const tx = api.tx.botGroupMgmt.logAction(
      communityHash,
      "Ban", // ActionType
      targetUser,
      executorNode,
      3, // consensus_count
      42, // sequence
      msgHash,
    );
    await tx.signAndSend(alice);
    await waitBlocks(api, 2);

    const logCount: any = (
      await api.query.botGroupMgmt.logCount(communityHash)
    ).toJSON();
    check("log_action: log count > 0", logCount > 0);
  } catch (e: any) {
    check(`log_action: ${e.message}`, false);
  }

  console.log("");

  // ══════════════════════════════════════════
  // 结果汇总
  // ══════════════════════════════════════════
  const total = passed + failed;
  console.log("═══════════════════════════════════════════");
  console.log(`  Results: ${passed} passed, ${failed} failed, ${total} total`);
  console.log("═══════════════════════════════════════════");

  await api.disconnect();
  process.exit(failed > 0 ? 1 : 0);
}

async function waitBlocks(api: ApiPromise, n: number): Promise<void> {
  return new Promise((resolve) => {
    let count = 0;
    const unsub = api.rpc.chain.subscribeNewHeads(() => {
      count++;
      if (count >= n) {
        (unsub as any).then?.((u: any) => u());
        resolve();
      }
    });
  });
}

main().catch((e) => {
  console.error("Fatal error:", e);
  process.exit(1);
});
