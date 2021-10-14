const { ApiPromise, WsProvider } = require('@polkadot/api');
const AWS = require('aws-sdk');
const CronJob = require('node-cron');
const Settings = require('./settings.json')

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress, types) {
    const provider = new WsProvider(nodeAddress);
    const api = await new ApiPromise({ provider, types });
    await api.isReady;
    return api;
}

async function main() {
    const api = await createPromiseApi(Settings.nodeAddress);

    AWS.config.update({region: Settings.region});
    const sns = new AWS.SNS({apiVersion: Settings.awsApiVersion});
    const topics = await sns.listTopics().promise();
    const topicArn = topics.Topics.find(t => t.TopicArn.includes(Settings.topicName)).TopicArn;

    let lastHeaderHex = Settings.lastHeaderHex;
    let lastNewBlockAt = Date.now();
    let sendingSns = false;
    const job = CronJob.schedule(Settings.cronTime, async () => {
        try {
            console.log(new Date().toISOString(), 'Checking for new block');
            const signedBlock = await api.rpc.chain.getBlock();
            const currentHeader = signedBlock.block.header.hash;

            if (currentHeader.toHex() != lastHeaderHex) {
                lastHeaderHex = currentHeader.toHex();
                lastNewBlockAt = Date.now();
                return;
            }
            console.log(`${Date.now()} , ${lastNewBlockAt}, ${Settings.requireNewBlockEveryMs}`)
            if (!sendingSns && (Date.now() - lastNewBlockAt > Settings.requireNewBlockEveryMs)) {
                sendingSns = true;
                let data = await sns.publish({
                    Message: `message: ${Settings.message};  Last header hex: ${lastHeaderHex}`,
                    TopicArn: topicArn,
                }).promise();
                console.warn(`Message ${Settings.message} sent to the topic ${topicArn}`);
                console.warn("MessageID is " + data.MessageId);
                sendingSns = false;
            }
        } catch (err) {
            sendingSns = false;
            lastNewBlockAt = Date.now();
            console.error(err, err.stack);
        }
    });
}

main().catch(console.error);
