import { Col, Row, Card, Button } from 'antd';
import React, { useEffect } from 'react';
// import { ConnectButton } from '../../components/ConnectButton';
import { useConnectionConfig } from '../../contexts/connection';
import { useMarkets } from '../../contexts/market';
import { Link } from 'react-router-dom';

// const stakeHammer = require('../../img/stakeHammer.svg');
const potionHero = require('../../img/potionHero.svg');
const friends = require('../../img/friends.svg');
const solanaLogo = require('../../img/solanaLogo.svg');
const polygonLogo = require('../../img/polygonDark.svg');
const keepLogo = require('../../img/keep.svg');
const hammer = require('../../img/hammer.svg');
const meat = require('../../img/meat.svg');
const ocean = require('../../img/ocean.svg');
const farmer = require('../../img/farmer.svg');
const stakeHammer = require('../../img/stakeHammer.svg');

export const HomeView = () => {
  const { marketEmitter, midPriceInUSD } = useMarkets();
  const { tokenMap } = useConnectionConfig();

  // let history = useHistory();
  // const handleClick = (to: string) => {
  //   history.push(to);
  // };

  useEffect(() => {
    const refreshTotal = () => {};

    const dispose = marketEmitter.onMarket(() => {
      refreshTotal();
    });

    refreshTotal();

    return () => {
      dispose();
    };
  }, [marketEmitter, midPriceInUSD, tokenMap]);

  return (
    <Row gutter={[48, 48]} align='middle'>
      <Col
        span={24}
        style={{
          marginTop: '70px',
          marginBottom: '50px',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
        }}
      >
        <h2 className='title' style={{ marginTop: '1em', marginBottom: 0 }}>
          The easiest way to stake your tokens
        </h2>
        <h2 className='title' style={{ marginTop: 0 }}>
          and get rewards
        </h2>
        <h3 className='subtext' style={{ marginBottom: '2em' }}>
          Enjoy automatic reinvestment of rewards and immediate access to staked
          tokens.
          {/* Making
            <span style={{ color: '#4E66DE', fontWeight: 900 }}>
              {' stake easier '}
            </span>
            to chew. */}
        </h3>
        <div style={{ position: 'relative' }}>
          <img id='meat' width='100' src={meat} alt='logo' />
          <img id='hammer' width='130' src={hammer} alt='logo' />
        </div>
      </Col>
      <Col
        span={22}
        offset={1}
        style={{
          marginTop: '120px',
          height: '450',
          display: 'flex',
          justifyContent: 'space-around',
        }}
      >
        <Card className='card' style={{ width: '30%' }}>
          <div
            style={{
              height: '100%',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <div>
              <img
                src={polygonLogo}
                alt='polygon logo'
                style={{ maxWidth: '70%' }}
              />
              <h2 className='subtext' style={{ marginTop: '5px' }}>
                POLYGON
              </h2>
            </div>
            <div style={{ marginTop: '5px', marginBottom: '5px' }}>
              <h2 className='title' style={{ marginBottom: 0 }}>
                10.2%
              </h2>
              <h3 style={{ marginBottom: 0 }}>Rewards</h3>
              <h5>(projected APY)</h5>
            </div>
            <Button className='tenderButton tenderButtonShade' disabled>
              Coming Soon
            </Button>
          </div>
        </Card>
        <Card className='card' style={{ width: '30%', height: '100%' }}>
          <div
            style={{
              height: '100%',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <div>
              <img
                src={solanaLogo}
                alt='solana logo'
                style={{ maxWidth: '70%' }}
              />
              <h2 className='subtext' style={{ marginTop: '5px' }}>
                SOLANA
              </h2>
            </div>
            <div style={{ marginTop: '5px', marginBottom: '5px' }}>
              <h2 className='title' style={{ marginBottom: 0 }}>
                15.5%
              </h2>
              <h3 style={{ marginBottom: 0 }}>Rewards</h3>
              <h5>(projected APY)</h5>
            </div>
            {/* <ConnectButton
              className='tenderButton tenderButtonShade'
              onClick={() => handleClick('/discover')}
            >
              Stake
            </ConnectButton> */}

            <Link to='/discover' style={{ width: '100%' }}>
              <Button className='tenderButton tenderButtonShade'>
                Discover
              </Button>
            </Link>
          </div>
        </Card>
        <Card className='card' style={{ width: '30%' }}>
          <div
            style={{
              height: '100%',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <div>
              <img
                src={keepLogo}
                alt='keep logo'
                style={{ maxWidth: '100%' }}
              />
              <h2 className='subtext' style={{ marginTop: '5px' }}>
                KEEP
              </h2>
            </div>
            <div style={{ marginTop: '5px', marginBottom: '5px' }}>
              <h2 className='title' style={{ marginBottom: 0 }}>
                9.8%
              </h2>
              <h3 style={{ marginBottom: 0 }}>Rewards</h3>
              <h5>(projected APY)</h5>
            </div>
            <Button className='tenderButton tenderButtonShade' disabled>
              Coming Soon
            </Button>
          </div>
        </Card>
      </Col>
      <Col span={10} offset={2}>
        <div style={{ marginTop: '100px', marginBottom: '70px' }}>
          <img style={{ maxWidth: '100%' }} src={potionHero} alt='potionHero' />
        </div>
        {/* <p>{balance}</p> */}
      </Col>
      <Col span={10}>
        <p className='subtext'> Tenderize is a liquid staking protocol. </p>
        <p className='subtext'>
          {' '}
          It allows you to enjoy your farm-fresh, tender staking returns without
          the wait of an unbonding period.
        </p>
      </Col>
      <Col span={24}>
        <div style={{ height: '65vh', marginBottom: '30px' }}>
          <div className='table card'>
            <h1 style={{ fontWeight: 600, marginTop: '20px' }}>
              Why Tenderize?
            </h1>
            <div className='table'>
              <div>
                <span
                  className='subtext table-left'
                  style={{ textAlign: 'center', width: '20%' }}
                >
                  <img width='50' src={ocean} alt='ocean icon' />
                </span>
                <p className='subtext'>
                  Liquify your stakes with tenderSol tokens.
                </p>
              </div>
              <div>
                <span
                  className='subtext table-left'
                  style={{ textAlign: 'center' }}
                >
                  <img
                    width='70'
                    src={stakeHammer}
                    alt='stake hammer icon'
                  ></img>
                </span>
                <p className='subtext'>
                  Make your staking process easy and tender.
                </p>
              </div>
              <div>
                <span
                  className='subtext table-left'
                  style={{ textAlign: 'center' }}
                >
                  <img width='50' src={farmer} alt='farmer icon' />
                </span>
                <p className='subtext'>Maximize your yield! No cool downs!</p>
              </div>
            </div>
          </div>
        </div>
      </Col>
      <Col span={24}>
        <div style={{ height: '65vh', marginBottom: '30px' }}>
          <div className='table card'>
            <h1 style={{ fontWeight: 600, marginTop: '20px' }}>
              How does it work?
            </h1>
            <div className='table'>
              <div>
                <span className='subtext table-left'>Step 1</span>
                <p className='subtext'>
                  Order off of our select, stake menu. Deposit your stake and
                  let the tenderizing begin.
                </p>
              </div>
              <div>
                <span className='subtext table-left'>Step 2</span>
                <p className='subtext'>
                  Receive freshly minted tender tokens that represent your stake
                  and rewards.
                </p>
              </div>
              <div>
                <span className='subtext table-left'>Step 3</span>
                <p className='subtext'>
                  Skip the waiting periods with your newly tenderized stakeq1.
                  Go utilize, liquidize, and collateralize, all while you
                  tenderize!
                </p>
              </div>
            </div>
          </div>
        </div>
      </Col>
      <Col span={12} offset={2}>
        <div style={{ marginTop: '80px', marginBottom: '100px' }}>
          <img style={{ maxWidth: '100%' }} src={friends} alt='friends' />
        </div>
      </Col>
      <Col span={8}>
        <p className='subtext'>
          Friends don’t let friends stake, unless they tenderize.
        </p>
      </Col>
      <Col span={24}>
        <div className='builton' />
      </Col>
    </Row>
  );
};
