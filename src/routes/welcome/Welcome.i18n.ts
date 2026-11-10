import {bundle} from '#lib/i18n/bundle';

export const welcome = bundle(
    {
        title: 'Sign in',
        server: 'Server address',
        email: 'Email',
        password: 'Password',
        connecting: 'Connecting...',
        signIn: 'Sign in',
        twoFactor: 'Two-step verification',
        twoFactorHint: 'Enter the 6-digit code from your authenticator',
        verifying: 'Verifying...',
        verify: 'Verify and sign in',
        errors: {
            server: 'Enter your Cloudreve server address',
            credentials: 'Enter your email and password',
            otp: 'Enter the 6-digit code',
        },
    },
    {
        title: '登录',
        server: '服务器地址',
        email: '邮箱',
        password: '密码',
        connecting: '正在连接...',
        signIn: '登录',
        twoFactor: '两步验证',
        twoFactorHint: '输入身份验证器中的 6 位验证码',
        verifying: '验证中...',
        verify: '验证并登录',
        errors: {
            server: '请输入 Cloudreve 服务器地址',
            credentials: '请输入邮箱和密码',
            otp: '请输入 6 位验证码',
        },
    },
);
