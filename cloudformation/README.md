# Deployment

```bash
aws cloudformation deploy --stack-name tarot-base --template-file base.yml --capabilities CAPABILITY_NAMED_IAM
../deploy.sh
aws cloudformation deploy --stack-name tarot-cloudfront --template-file cloudfront.yml
```
