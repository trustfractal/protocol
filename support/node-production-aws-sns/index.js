const { ApiPromise, WsProvider } = require('@polkadot/api');
const AWS = require('aws-sdk');
const Settings = require('./settings.json')
const types = require('../../blockchain/types.json')
var argv = require('minimist')(process.argv.slice(2));

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress) {
    const provider = new WsProvider(nodeAddress);
    const api = await new ApiPromise({ provider, types });
    await api.isReady;
    return api;
}

async function main() {
    const api = await createPromiseApi(argv.nodeAddress || Settings.nodeAddress);

    AWS.config.update({region: Settings.region});
    const sns = new AWS.SNS({apiVersion: Settings.awsApiVersion});
    const topics = await sns.listTopics().promise();
    const topicArn = topics.Topics.find(t => t.TopicArn.includes(Settings.topicName)).TopicArn;

    let lastHeaderHex = Settings.lastHeaderHex;
    let lastNewBlockAt = Date.now();
    while (true) {
        try {
            await new Promise(r => setTimeout(r, Settings.sleepIntervalMs));
            const signedBlock = await api.rpc.chain.getBlock();
            const currentHeader = signedBlock.block.header.hash;

            if (currentHeader.toHex() != lastHeaderHex) {
                lastHeaderHex = currentHeader.toHex();
                lastNewBlockAt = Date.now();
                console.log(`Block header hex: ${lastHeaderHex} at ${new Date().toISOString()}`);
                continue;
            }

            console.log(new Date().toISOString(), `Have not seen new block for ${new Date() - lastNewBlockAt}ms`);

            if (Date.now() - lastNewBlockAt > Settings.requireNewBlockEveryMs) {
                sendingSns = true;
                let data = await sns.publish({
                    Message: `message: ${Settings.message};  Last header hex: ${lastHeaderHex}`,
                    TopicArn: topicArn,
                }).promise();
                console.warn(`Message ${Settings.message} sent to the topic ${topicArn}`);
                console.warn("MessageID is " + data.MessageId);
                await new Promise(r => setTimeout(r, 1000 * 60 * Settings.minutesUntilNextCheckAfterAlarm));
            }
        } catch (err) {
            lastNewBlockAt = Date.now();
            console.error(err, err.stack);
        }
    };
}

main().catch(console.error);
